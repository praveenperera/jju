use ahash::HashMap;
use colored::{Color, Colorize};
use duct::cmd;
use eyre::{Context as _, Result};
use jju_core::split_hunk::{LineRange, SplitSelectionPlan};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct SplitHunkOptions {
    pub message: Option<String>,
    pub revision: String,
    pub file_filter: Option<String>,
    pub lines: Option<String>,
    pub hunks: Option<String>,
    pub pattern: Option<String>,
    pub preview: bool,
    pub invert: bool,
    pub dry_run: bool,
}

impl SplitHunkOptions {
    fn commit_message(&self) -> Result<&str> {
        if self.preview {
            return Ok("");
        }

        self.message
            .as_deref()
            .ok_or_else(|| eyre::eyre!("--message is required unless using --preview"))
    }
}

#[derive(Debug, Clone)]
pub struct SplitHunkCommand {
    options: SplitHunkOptions,
}

impl SplitHunkCommand {
    pub fn new(options: SplitHunkOptions) -> Self {
        Self { options }
    }

    pub fn run(self) -> Result<()> {
        SplitHunkWorkflow::new(self.options).run()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffLineKind {
    Context,
    Added,
    Removed,
}

impl DiffLineKind {
    fn is_added(self) -> bool {
        matches!(self, Self::Added)
    }

    fn is_removed(self) -> bool {
        matches!(self, Self::Removed)
    }
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
    fn new(old_start: usize, old_count: usize, new_start: usize, new_count: usize) -> Self {
        Self {
            old_start,
            old_count,
            new_start,
            new_count,
            lines: Vec::new(),
        }
    }

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

impl FileDiff {
    fn new(path: String) -> Self {
        Self {
            path,
            hunks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedHunk {
    file_index: usize,
    hunk_index: usize,
}

struct SplitHunkWorkflow {
    options: SplitHunkOptions,
}

impl SplitHunkWorkflow {
    fn new(options: SplitHunkOptions) -> Self {
        Self { options }
    }

    fn run(self) -> Result<()> {
        let files = self.load_files()?;
        if files.is_empty() {
            return Ok(());
        }

        if self.options.preview {
            preview_hunks(&files);
            return Ok(());
        }

        let selection = self.build_selection_plan()?;
        let selected = select_hunks(&files, &selection);
        if selected.is_empty() {
            println!("{}", "No hunks matched selection criteria".yellow());
            return Ok(());
        }

        println!(
            "{} {} hunks",
            "Selected".green(),
            selected.len().to_string().cyan()
        );

        let new_contents = apply_selected_hunks(&files, &selected, &self.options.revision)?;
        if self.options.dry_run {
            println!("\n{}", "Dry run - would commit:".yellow());
            for path in new_contents.keys() {
                println!("  {}", path.cyan());
            }
            return Ok(());
        }

        let message = self.options.commit_message()?;
        let original_contents = collect_original_contents(&files, &self.options.revision);
        execute_split_hunk(
            &self.options.revision,
            message,
            &new_contents,
            &original_contents,
        )?;

        println!("{} {}", "Created split commit:".green(), message.cyan());
        Ok(())
    }

    fn load_files(&self) -> Result<Vec<FileDiff>> {
        let diff_output = cmd!("jj", "diff", "-r", &self.options.revision, "--git")
            .stdout_capture()
            .stderr_capture()
            .read()
            .wrap_err("failed to get diff")?;

        if diff_output.is_empty() {
            println!("{}", "No changes in revision".yellow());
            return Ok(Vec::new());
        }

        let mut files = parse_diff_output(&diff_output);
        if let Some(filter) = self.options.file_filter.as_deref() {
            files.retain(|file| file.path.contains(filter));
        }

        if files.is_empty() {
            println!("{}", "No matching files found".yellow());
        }

        Ok(files)
    }

    fn build_selection_plan(&self) -> Result<SplitSelectionPlan> {
        Ok(SplitSelectionPlan {
            hunk_indices: self
                .options
                .hunks
                .as_deref()
                .map(parse_hunk_indices)
                .transpose()?,
            line_ranges: self
                .options
                .lines
                .as_deref()
                .map(parse_line_ranges)
                .transpose()?,
            pattern: self.options.pattern.clone(),
            invert: self.options.invert,
        })
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

fn parse_diff_output(diff_output: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;

    for line in diff_output.lines() {
        if line.starts_with("diff --git ") {
            flush_current_hunk(&mut current_file, &mut current_hunk);
            flush_current_file(&mut files, &mut current_file);
            current_file = Some(FileDiff::new(parse_file_path(line)));
            continue;
        }

        if line.starts_with("@@ ") {
            flush_current_hunk(&mut current_file, &mut current_hunk);
            let (old_start, old_count, new_start, new_count) = parse_hunk_header(line);
            current_hunk = Some(DiffHunk::new(old_start, old_count, new_start, new_count));
            continue;
        }

        if let Some(hunk) = &mut current_hunk
            && let Some(diff_line) = parse_diff_line(line)
        {
            hunk.lines.push(diff_line);
        }
    }

    flush_current_hunk(&mut current_file, &mut current_hunk);
    flush_current_file(&mut files, &mut current_file);
    files
}

fn parse_line_ranges(input: &str) -> Result<Vec<LineRange>> {
    let mut ranges = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut split = part.split('-');
            let start: usize = split
                .next()
                .ok_or_else(|| eyre::eyre!("invalid range: {part}"))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range start: {part}"))?;
            let end: usize = split
                .next()
                .ok_or_else(|| eyre::eyre!("invalid range: {part}"))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range end: {part}"))?;
            ranges.push(LineRange(start, end));
        } else {
            let line: usize = part
                .parse()
                .wrap_err_with(|| format!("invalid line: {part}"))?;
            ranges.push(LineRange(line, line));
        }
    }
    Ok(ranges)
}

fn parse_hunk_indices(input: &str) -> Result<Vec<usize>> {
    input
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .wrap_err_with(|| format!("invalid hunk index: {part}"))
        })
        .collect()
}

fn parse_file_path(line: &str) -> String {
    line.split_whitespace()
        .nth(3)
        .map(|part| part.trim_start_matches("b/"))
        .unwrap_or("")
        .to_string()
}

fn parse_diff_line(line: &str) -> Option<DiffLine> {
    let (kind, content) = if let Some(content) = line.strip_prefix('+') {
        (DiffLineKind::Added, content.to_string())
    } else if let Some(content) = line.strip_prefix('-') {
        (DiffLineKind::Removed, content.to_string())
    } else if let Some(content) = line.strip_prefix(' ') {
        (DiffLineKind::Context, content.to_string())
    } else {
        return None;
    };

    Some(DiffLine { kind, content })
}

fn parse_hunk_header(line: &str) -> (usize, usize, usize, usize) {
    static HUNK_HEADER_RE: OnceLock<Option<Regex>> = OnceLock::new();
    let Some(regex) = HUNK_HEADER_RE
        .get_or_init(|| Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").ok())
        .as_ref()
    else {
        return (1, 1, 1, 1);
    };

    let Some(captures) = regex.captures(line) else {
        return (1, 1, 1, 1);
    };

    let old_start = captures
        .get(1)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    let old_count = captures
        .get(2)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    let new_start = captures
        .get(3)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    let new_count = captures
        .get(4)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    (old_start, old_count, new_start, new_count)
}

fn flush_current_hunk(current_file: &mut Option<FileDiff>, current_hunk: &mut Option<DiffHunk>) {
    if let Some(hunk) = current_hunk.take()
        && let Some(file) = current_file
    {
        file.hunks.push(hunk);
    }
}

fn flush_current_file(files: &mut Vec<FileDiff>, current_file: &mut Option<FileDiff>) {
    if let Some(file) = current_file.take() {
        files.push(file);
    }
}

fn categorize_hunk(hunk: &DiffHunk) -> (&'static str, Color) {
    let has_added = hunk.lines.iter().any(|line| line.kind.is_added());
    let has_removed = hunk.lines.iter().any(|line| line.kind.is_removed());

    match (has_added, has_removed) {
        (true, true) => ("modified", Color::Yellow),
        (true, false) => ("added", Color::Green),
        (false, true) => ("removed", Color::Red),
        (false, false) => ("context", Color::White),
    }
}

fn select_hunks(files: &[FileDiff], selection: &SplitSelectionPlan) -> Vec<SelectedHunk> {
    let pattern = selection
        .pattern
        .as_deref()
        .map(Regex::new)
        .transpose()
        .ok()
        .flatten();
    let mut selected = Vec::new();
    let mut global_idx = 0;

    for (file_index, file) in files.iter().enumerate() {
        for (hunk_index, hunk) in file.hunks.iter().enumerate() {
            let mut matches = matches_selection(hunk, global_idx, selection, pattern.as_ref());
            if selection.invert {
                matches = !matches;
            }

            if matches {
                selected.push(SelectedHunk {
                    file_index,
                    hunk_index,
                });
            }

            global_idx += 1;
        }
    }

    selected
}

fn matches_selection(
    hunk: &DiffHunk,
    global_idx: usize,
    selection: &SplitSelectionPlan,
    pattern: Option<&Regex>,
) -> bool {
    if selection.matches_all() {
        return true;
    }

    if let Some(indices) = selection.hunk_indices.as_deref()
        && indices.contains(&global_idx)
    {
        return true;
    }

    if let Some(ranges) = selection.line_ranges.as_deref()
        && hunk_overlaps_lines(hunk, ranges)
    {
        return true;
    }

    if let Some(pattern) = pattern
        && hunk_matches_pattern(hunk, pattern)
    {
        return true;
    }

    false
}

fn hunk_overlaps_lines(hunk: &DiffHunk, ranges: &[LineRange]) -> bool {
    let hunk_start = hunk.first_line();
    let hunk_end = hunk.last_line();
    ranges
        .iter()
        .any(|LineRange(start, end)| hunk_start <= *end && hunk_end >= *start)
}

fn hunk_matches_pattern(hunk: &DiffHunk, pattern: &Regex) -> bool {
    hunk.lines
        .iter()
        .any(|line| pattern.is_match(&line.content))
}

fn apply_selected_hunks(
    files: &[FileDiff],
    selected: &[SelectedHunk],
    revision: &str,
) -> Result<HashMap<String, String>> {
    let mut results = HashMap::default();
    let mut file_hunks: HashMap<usize, Vec<usize>> = HashMap::default();

    for selected_hunk in selected {
        file_hunks
            .entry(selected_hunk.file_index)
            .or_default()
            .push(selected_hunk.hunk_index);
    }

    for (file_index, hunk_indices) in file_hunks {
        let file = &files[file_index];
        let parent_rev = format!("{}-", revision);
        let parent_lines = read_file_lines_or_empty(&parent_rev, &file.path);
        let result_lines = apply_hunks_to_lines(&parent_lines, &file.hunks, &hunk_indices);
        results.insert(file.path.clone(), result_lines.join("\n"));
    }

    Ok(results)
}

fn apply_hunks_to_lines(
    parent_lines: &[String],
    hunks: &[DiffHunk],
    selected_indices: &[usize],
) -> Vec<String> {
    let mut result_lines = parent_lines.to_vec();
    let mut sorted_indices = selected_indices.to_vec();
    sorted_indices.sort_by(|left, right| right.cmp(left));

    for hunk_index in sorted_indices {
        let hunk = &hunks[hunk_index];
        let insert_pos = hunk.old_start.saturating_sub(1);
        let remove_end = (insert_pos + hunk.old_count).min(result_lines.len());
        result_lines.drain(insert_pos..remove_end);

        let new_lines: Vec<String> = hunk
            .lines
            .iter()
            .filter(|line| !line.kind.is_removed())
            .map(|line| line.content.clone())
            .collect();

        for (index, line) in new_lines.into_iter().enumerate() {
            result_lines.insert(insert_pos + index, line);
        }
    }

    result_lines
}

fn read_file_lines_or_empty(revision: &str, path: &str) -> Vec<String> {
    cmd!("jj", "file", "show", "-r", revision, path)
        .stdout_capture()
        .stderr_capture()
        .read()
        .unwrap_or_default()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

fn collect_original_contents(files: &[FileDiff], revision: &str) -> HashMap<String, String> {
    files.iter().fold(HashMap::default(), |mut contents, file| {
        let content = cmd!("jj", "file", "show", "-r", revision, &file.path)
            .stdout_capture()
            .stderr_capture()
            .read()
            .unwrap_or_default();
        contents.insert(file.path.clone(), content);
        contents
    })
}

fn execute_split_hunk(
    revision: &str,
    message: &str,
    new_contents: &HashMap<String, String>,
    original_contents: &HashMap<String, String>,
) -> Result<()> {
    let parent_rev = format!("{}-", revision);
    cmd!("jj", "new", &parent_rev)
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to create new commit")?;

    for (path, content) in new_contents {
        std::fs::write(path, content).wrap_err_with(|| format!("failed to write {path}"))?;
    }

    cmd!("jj", "describe", "-m", message)
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to set commit message")?;

    let split_change_id = cmd!(
        "jj",
        "log",
        "-r",
        "@",
        "--no-graph",
        "-T",
        "change_id.short()"
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .wrap_err("failed to get split commit change id")?;

    cmd!("jj", "rebase", "-s", revision, "-d", "@")
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to rebase original revision")?;

    cmd!("jj", "edit", revision)
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to edit original revision")?;

    for (path, content) in original_contents {
        std::fs::write(path, content).wrap_err_with(|| format!("failed to restore {path}"))?;
    }

    cmd!("jj", "edit", split_change_id.trim())
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to return to split commit")?;

    Ok(())
}
