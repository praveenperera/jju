use crate::jj_lib_helpers::JjRepo;
use ahash::{HashMap, HashMapExt};
use clap::{Parser, Subcommand};
use colored::Colorize;
use duct::cmd;
use eyre::{bail, Context as _, Result};
use log::debug;
use regex::Regex;
use std::ffi::OsString;
use std::io::Write;

#[derive(Debug, Clone, Parser)]
#[command(name = "jju", author, version, about, styles = crate::cli::get_styles())]
pub struct Jj {
    #[command(subcommand)]
    pub subcommand: Option<JjCmd>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum JjCmd {
    /// Sync the current stack with remote trunk (master/main/trunk)
    #[command(visible_alias = "ss")]
    StackSync {
        /// Push the first bookmark after syncing
        #[arg(short, long)]
        push: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Display the current stack as a tree
    #[command(visible_alias = "t")]
    Tree {
        /// Show all commits, including those without bookmarks
        #[arg(short, long)]
        full: bool,

        /// Base revision to start the tree from (default: trunk())
        #[arg(long)]
        from: Option<String>,
    },

    /// Split hunks from a commit non-interactively
    #[command(visible_alias = "sh")]
    SplitHunk {
        /// Commit message for the new commit (required unless --preview)
        #[arg(short, long)]
        message: Option<String>,

        /// Revision to split (default: @)
        #[arg(short, long, default_value = "@")]
        revision: String,

        /// File to select hunks from
        #[arg(long)]
        file: Option<String>,

        /// Line ranges to include (e.g., "10-20,30-40")
        #[arg(long)]
        lines: Option<String>,

        /// Hunk indices to include (e.g., "0,2,5")
        #[arg(long)]
        hunks: Option<String>,

        /// Regex pattern to match hunk content
        #[arg(long)]
        pattern: Option<String>,

        /// Preview hunks with indices (don't split)
        #[arg(long)]
        preview: bool,

        /// Exclude matched hunks instead of including them
        #[arg(long)]
        invert: bool,

        /// Show what would be committed without committing
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(args: &[OsString]) -> Result<()> {
    debug!("jj args: {args:?}");
    let flags = Jj::parse_from(args);
    run_with_flags(flags)
}

pub fn run_with_flags(flags: Jj) -> Result<()> {
    match flags.subcommand {
        None => crate::cmd::jj_tui::run(),
        Some(JjCmd::StackSync { push, force }) => stack_sync(push, force),
        Some(JjCmd::Tree { full, from }) => tree(full, from),
        Some(JjCmd::SplitHunk {
            message,
            revision,
            file,
            lines,
            hunks,
            pattern,
            preview,
            invert,
            dry_run,
        }) => split_hunk(
            message, revision, file, lines, hunks, pattern, preview, invert, dry_run,
        ),
    }
}

/// Detect the trunk branch name (master, main, or trunk)
fn detect_trunk_branch() -> Result<String> {
    let output = cmd!(
        "jj",
        "log",
        "-r",
        "trunk()",
        "--no-graph",
        "-T",
        "local_bookmarks",
        "--limit",
        "1"
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .wrap_err("failed to detect trunk branch")?;

    let trunk = output
        .split_whitespace()
        .next()
        .unwrap_or("master")
        .to_string();

    Ok(trunk)
}

fn stack_sync(push: bool, force: bool) -> Result<()> {
    println!("{}", "Fetching from remote...".dimmed());
    cmd!("jj", "git", "fetch")
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to fetch")?;

    // detect and sync trunk bookmark
    let trunk = detect_trunk_branch()?;
    println!("{}{}", "Syncing ".dimmed(), trunk);
    cmd!(
        "jj",
        "bookmark",
        "set",
        &trunk,
        "-r",
        format!("{trunk}@origin")
    )
    .stdout_null()
    .stderr_null()
    .run()
    .wrap_err("failed to sync trunk bookmark")?;

    // find the root(s) of the stack
    let roots_revset = format!("roots({trunk}..@)");
    let roots_output = cmd!(
        "jj",
        "log",
        "-r",
        &roots_revset,
        "--no-graph",
        "-T",
        r#"change_id.short() ++ "\n""#
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .wrap_err("failed to find stack roots")?;

    let roots: Vec<&str> = roots_output.lines().filter(|l| !l.is_empty()).collect();
    debug!("stack roots: {:?}", roots);

    if roots.is_empty() {
        println!(
            "{}{}{}",
            "No commits after ".dimmed(),
            trunk,
            ", nothing to rebase".dimmed()
        );
        return Ok(());
    }

    // show confirmation unless --force
    if !force {
        println!(
            "Will rebase the following commits on top of {}:",
            trunk.cyan()
        );
        for root in &roots {
            let desc = cmd!(
                "jj",
                "log",
                "-r",
                root,
                "--no-graph",
                "-T",
                "description.first_line()"
            )
            .stdout_capture()
            .stderr_capture()
            .read()
            .unwrap_or_default();
            println!("  {}  {}", root.purple(), desc.dimmed());
            println!(
                "  {}",
                format!("jj rebase --source (-s) {root} --onto (-o) {trunk} --skip-emptied")
                    .dimmed()
            );
        }
        print!("Continue? [y/N] ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted".yellow());
            return Ok(());
        }
    }

    // rebase from each root (usually just one)
    // --skip-emptied handles merged commits by abandoning ones that became empty
    for root in &roots {
        println!("{}{}...", "Rebasing stack from ".dimmed(), root);
        cmd!(
            "jj",
            "rebase",
            "--source",
            root,
            "--onto",
            &trunk,
            "--skip-emptied"
        )
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err_with(|| format!("failed to rebase from {root}"))?;
    }

    // clean up bookmarks marked as deleted on remote (after rebase so --skip-emptied can work)
    let tracked = cmd!("jj", "bookmark", "list", "--tracked")
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to list tracked bookmarks")?;

    for line in tracked.lines() {
        if line.contains("[deleted]")
            && let Some(bookmark) = line.split_whitespace().next()
        {
            println!("{}{}", "Deleting merged bookmark: ".dimmed(), bookmark);
            cmd!("jj", "bookmark", "delete", bookmark)
                .stdout_null()
                .stderr_null()
                .run()
                .wrap_err_with(|| format!("failed to delete bookmark {bookmark}"))?;
        }
    }

    if push {
        // find and push the first bookmark in the rebased stack
        let revset = format!("({trunk}..@) & bookmarks()");
        let template = r#"bookmarks ++ "\n""#;
        let output = cmd!(
            "jj",
            "log",
            "-r",
            &revset,
            "--reversed",
            "--no-graph",
            "-T",
            template,
            "--limit",
            "1"
        )
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to get bookmarks")?;

        if let Some(bookmark) = output.lines().find(|l| !l.is_empty()) {
            let bookmark = bookmark.trim();
            println!("{}{}...", "Pushing ".dimmed(), bookmark);
            cmd!("jj", "git", "push", "--bookmark", bookmark)
                .stdout_null()
                .stderr_null()
                .run()
                .wrap_err("failed to push")?;
        } else {
            println!("{}", "No bookmarks found to push".dimmed());
        }
    }

    println!("{}", "Stack sync complete".green());
    Ok(())
}

fn tree(full: bool, from: Option<String>) -> Result<()> {
    use crate::cmd::jj_tui::{cli_tree, tree::TreeState};

    let jj_repo = JjRepo::load(None)?;
    let base = from.as_deref().unwrap_or("trunk()");
    let tree = TreeState::load_with_base(&jj_repo, base)?;
    cli_tree::print_tree(&tree, full);
    Ok(())
}

// --- Split Hunk Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
struct DiffLine {
    kind: DiffLineKind,
    content: String,
}

#[derive(Debug, Clone)]
struct DiffHunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffLine>,
}

impl DiffHunk {
    fn first_line(&self) -> usize {
        self.new_start
    }

    fn last_line(&self) -> usize {
        self.new_start + self.new_count.saturating_sub(1)
    }
}

#[derive(Debug, Clone)]
struct FileDiff {
    path: String,
    hunks: Vec<DiffHunk>,
}

// --- Split Hunk Helpers ---

fn parse_diff_output(diff_output: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;

    for line in diff_output.lines() {
        if line.starts_with("diff --git ") {
            // save previous hunk and file
            if let Some(hunk) = current_hunk.take()
                && let Some(ref mut file) = current_file
            {
                file.hunks.push(hunk);
            }
            if let Some(file) = current_file.take() {
                files.push(file);
            }
            // extract file path from "diff --git a/path b/path"
            let parts: Vec<&str> = line.split_whitespace().collect();
            let path = parts
                .get(3)
                .map(|s| s.trim_start_matches("b/"))
                .unwrap_or("")
                .to_string();
            current_file = Some(FileDiff {
                path,
                hunks: Vec::new(),
            });
        } else if line.starts_with("@@ ") {
            // save previous hunk
            if let Some(hunk) = current_hunk.take()
                && let Some(ref mut file) = current_file
            {
                file.hunks.push(hunk);
            }
            // parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            let hunk_info = parse_hunk_header(line);
            current_hunk = Some(DiffHunk {
                old_start: hunk_info.0,
                old_count: hunk_info.1,
                new_start: hunk_info.2,
                new_count: hunk_info.3,
                lines: Vec::new(),
            });
        } else if let Some(ref mut hunk) = current_hunk {
            let (kind, content) = if let Some(content) = line.strip_prefix('+') {
                (DiffLineKind::Added, content.to_string())
            } else if let Some(content) = line.strip_prefix('-') {
                (DiffLineKind::Removed, content.to_string())
            } else if let Some(content) = line.strip_prefix(' ') {
                (DiffLineKind::Context, content.to_string())
            } else {
                continue;
            };

            hunk.lines.push(DiffLine { kind, content });
        }
    }

    // save final hunk and file
    if let Some(hunk) = current_hunk
        && let Some(ref mut file) = current_file
    {
        file.hunks.push(hunk);
    }
    if let Some(file) = current_file {
        files.push(file);
    }

    files
}

fn parse_hunk_header(line: &str) -> (usize, usize, usize, usize) {
    // @@ -old_start,old_count +new_start,new_count @@
    let re = Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").unwrap();
    if let Some(caps) = re.captures(line) {
        let old_start = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
        let old_count = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
        let new_start = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
        let new_count = caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
        (old_start, old_count, new_start, new_count)
    } else {
        (1, 1, 1, 1)
    }
}

fn categorize_hunk(hunk: &DiffHunk) -> (&'static str, colored::Color) {
    let has_added = hunk.lines.iter().any(|l| l.kind == DiffLineKind::Added);
    let has_removed = hunk.lines.iter().any(|l| l.kind == DiffLineKind::Removed);

    match (has_added, has_removed) {
        (true, true) => ("modified", colored::Color::Yellow),
        (true, false) => ("added", colored::Color::Green),
        (false, true) => ("removed", colored::Color::Red),
        (false, false) => ("context", colored::Color::White),
    }
}

fn preview_hunks(files: &[FileDiff]) {
    let mut global_idx = 0;
    for file in files {
        println!("\n{}", file.path.cyan().bold());
        for hunk in &file.hunks {
            let (label, color) = categorize_hunk(hunk);
            println!(
                "\n  {} {} (lines {}-{})",
                format!("[{}]", global_idx).white().bold(),
                label.color(color),
                hunk.first_line(),
                hunk.last_line()
            );
            for line in &hunk.lines {
                let prefix = match line.kind {
                    DiffLineKind::Context => " ".white(),
                    DiffLineKind::Added => "+".green(),
                    DiffLineKind::Removed => "-".red(),
                };
                let content = match line.kind {
                    DiffLineKind::Context => line.content.white(),
                    DiffLineKind::Added => line.content.green(),
                    DiffLineKind::Removed => line.content.red(),
                };
                println!("    {}{}", prefix, content);
            }
            global_idx += 1;
        }
    }
}

fn parse_line_ranges(input: &str) -> Result<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut split = part.split('-');
            let start: usize = split
                .next()
                .ok_or_else(|| eyre::eyre!("invalid range: {}", part))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range start: {}", part))?;
            let end: usize = split
                .next()
                .ok_or_else(|| eyre::eyre!("invalid range: {}", part))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range end: {}", part))?;
            ranges.push((start, end));
        } else {
            let line: usize = part.parse().wrap_err_with(|| format!("invalid line: {}", part))?;
            ranges.push((line, line));
        }
    }
    Ok(ranges)
}

fn parse_hunk_indices(input: &str) -> Result<Vec<usize>> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<usize>()
                .wrap_err_with(|| format!("invalid hunk index: {}", s))
        })
        .collect()
}

fn hunk_overlaps_lines(hunk: &DiffHunk, ranges: &[(usize, usize)]) -> bool {
    let hunk_start = hunk.first_line();
    let hunk_end = hunk.last_line();
    ranges
        .iter()
        .any(|(start, end)| hunk_start <= *end && hunk_end >= *start)
}

fn hunk_matches_pattern(hunk: &DiffHunk, pattern: &Regex) -> bool {
    hunk.lines.iter().any(|line| pattern.is_match(&line.content))
}

fn select_hunks(
    files: &[FileDiff],
    hunk_indices: Option<&[usize]>,
    line_ranges: Option<&[(usize, usize)]>,
    pattern: Option<&Regex>,
    invert: bool,
) -> Vec<(usize, usize)> {
    // returns (file_idx, hunk_idx) pairs
    let mut selected = Vec::new();
    let mut global_idx = 0;

    for (file_idx, file) in files.iter().enumerate() {
        for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
            let mut matches = false;

            if let Some(indices) = hunk_indices
                && indices.contains(&global_idx)
            {
                matches = true;
            }

            if let Some(ranges) = line_ranges
                && hunk_overlaps_lines(hunk, ranges)
            {
                matches = true;
            }

            if let Some(re) = pattern
                && hunk_matches_pattern(hunk, re)
            {
                matches = true;
            }

            // if no criteria specified, select all
            if hunk_indices.is_none() && line_ranges.is_none() && pattern.is_none() {
                matches = true;
            }

            if invert {
                matches = !matches;
            }

            if matches {
                selected.push((file_idx, hunk_idx));
            }

            global_idx += 1;
        }
    }

    selected
}

fn apply_selected_hunks(
    files: &[FileDiff],
    selected: &[(usize, usize)],
    revision: &str,
) -> Result<HashMap<String, String>> {
    // for each file, get parent content and apply selected hunks
    let mut results = HashMap::new();

    // group selected hunks by file
    let mut file_hunks: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(file_idx, hunk_idx) in selected {
        file_hunks.entry(file_idx).or_default().push(hunk_idx);
    }

    for (file_idx, hunk_indices) in file_hunks {
        let file = &files[file_idx];

        // get parent content
        let parent_rev = format!("{}-", revision);
        let parent_content = cmd!("jj", "file", "show", "-r", &parent_rev, &file.path)
            .stdout_capture()
            .stderr_capture()
            .read()
            .unwrap_or_default();

        let parent_lines: Vec<&str> = parent_content.lines().collect();
        let mut result_lines: Vec<String> = parent_lines.iter().map(|s| s.to_string()).collect();

        // sort hunk indices in reverse order so we can apply from bottom to top
        let mut sorted_indices = hunk_indices;
        sorted_indices.sort_by(|a, b| b.cmp(a));

        for hunk_idx in sorted_indices {
            let hunk = &file.hunks[hunk_idx];
            let insert_pos = hunk.old_start.saturating_sub(1);

            // remove old lines
            let remove_end = (insert_pos + hunk.old_count).min(result_lines.len());
            result_lines.drain(insert_pos..remove_end);

            // insert new lines (added + context from the hunk)
            let new_lines: Vec<String> = hunk
                .lines
                .iter()
                .filter(|l| l.kind != DiffLineKind::Removed)
                .map(|l| l.content.clone())
                .collect();

            for (i, line) in new_lines.into_iter().enumerate() {
                result_lines.insert(insert_pos + i, line);
            }
        }

        results.insert(file.path.clone(), result_lines.join("\n"));
    }

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn split_hunk(
    message: Option<String>,
    revision: String,
    file_filter: Option<String>,
    lines: Option<String>,
    hunks: Option<String>,
    pattern: Option<String>,
    preview: bool,
    invert: bool,
    dry_run: bool,
) -> Result<()> {
    // require message unless preview
    if !preview && message.is_none() {
        bail!("--message is required unless using --preview");
    }

    // get diff for the revision
    let diff_output = cmd!("jj", "diff", "-r", &revision, "--git")
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to get diff")?;

    if diff_output.is_empty() {
        println!("{}", "No changes in revision".yellow());
        return Ok(());
    }

    let mut files = parse_diff_output(&diff_output);

    // filter by file if specified
    if let Some(ref filter) = file_filter {
        files.retain(|f| f.path.contains(filter));
    }

    if files.is_empty() {
        println!("{}", "No matching files found".yellow());
        return Ok(());
    }

    // preview mode
    if preview {
        preview_hunks(&files);
        return Ok(());
    }

    // parse selection criteria
    let hunk_indices = hunks.as_ref().map(|h| parse_hunk_indices(h)).transpose()?;
    let line_ranges = lines.as_ref().map(|l| parse_line_ranges(l)).transpose()?;
    let pattern_re = pattern.as_ref().map(|p| Regex::new(p)).transpose()?;

    // select hunks
    let selected = select_hunks(
        &files,
        hunk_indices.as_deref(),
        line_ranges.as_deref(),
        pattern_re.as_ref(),
        invert,
    );

    if selected.is_empty() {
        println!("{}", "No hunks matched selection criteria".yellow());
        return Ok(());
    }

    // show what will be committed
    println!(
        "{} {} hunks",
        "Selected".green(),
        selected.len().to_string().cyan()
    );

    // apply selected hunks to get new file contents
    let new_contents = apply_selected_hunks(&files, &selected, &revision)?;

    if dry_run {
        println!("\n{}", "Dry run - would commit:".yellow());
        for path in new_contents.keys() {
            println!("  {}", path.cyan());
        }
        return Ok(());
    }

    // create a new commit with selected changes
    let message = message.unwrap();

    // create new commit from parent
    let parent_rev = format!("{}-", revision);
    cmd!("jj", "new", &parent_rev)
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to create new commit")?;

    // write the new file contents
    for (path, content) in &new_contents {
        std::fs::write(path, content).wrap_err_with(|| format!("failed to write {}", path))?;
    }

    // set the commit message
    cmd!("jj", "describe", "-m", &message)
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to set commit message")?;

    // rebase the original revision on top
    cmd!("jj", "rebase", "-r", &revision, "-d", "@")
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to rebase original revision")?;

    println!(
        "{} {}",
        "Created split commit:".green(),
        message.cyan()
    );

    Ok(())
}
