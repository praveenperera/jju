use colored::Colorize;
use duct::cmd;
use eyre::{Context as _, Result};
use log::debug;
use std::io::Write;

pub(crate) struct StackSyncCommand {
    push: bool,
    force: bool,
}

impl StackSyncCommand {
    pub(crate) fn new(push: bool, force: bool) -> Self {
        Self { push, force }
    }

    pub(crate) fn run(self) -> Result<()> {
        println!("{}", "Fetching from remote...".dimmed());
        cmd!("jj", "git", "fetch")
            .stdout_null()
            .stderr_null()
            .run()
            .wrap_err("failed to fetch")?;

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

        let roots = stack_roots(&trunk)?;
        if roots.is_empty() {
            println!(
                "{}{}{}",
                "No commits after ".dimmed(),
                trunk,
                ", nothing to rebase".dimmed()
            );
            return Ok(());
        }

        if !self.force && !confirm_rebase(&trunk, &roots)? {
            println!("{}", "Aborted".yellow());
            return Ok(());
        }

        rebase_roots(&trunk, &roots)?;
        cleanup_deleted_bookmarks()?;

        if self.push {
            push_first_bookmark(&trunk)?;
        }

        println!("{}", "Stack sync complete".green());
        Ok(())
    }
}

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

    Ok(output
        .split_whitespace()
        .next()
        .unwrap_or("master")
        .to_string())
}

fn stack_roots(trunk: &str) -> Result<Vec<String>> {
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

    let roots: Vec<String> = roots_output
        .lines()
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    debug!("stack roots: {:?}", roots);
    Ok(roots)
}

fn confirm_rebase(trunk: &str, roots: &[String]) -> Result<bool> {
    println!(
        "Will rebase the following commits on top of {}:",
        trunk.cyan()
    );

    for root in roots {
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
            format!("jj rebase --source (-s) {root} --onto (-o) {trunk} --skip-emptied").dimmed()
        );
    }

    print!("Continue? [y/N] ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn rebase_roots(trunk: &str, roots: &[String]) -> Result<()> {
    for root in roots {
        println!("{}{}...", "Rebasing stack from ".dimmed(), root);
        cmd!(
            "jj",
            "rebase",
            "--source",
            root,
            "--onto",
            trunk,
            "--skip-emptied"
        )
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err_with(|| format!("failed to rebase from {root}"))?;
    }

    Ok(())
}

fn cleanup_deleted_bookmarks() -> Result<()> {
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

    Ok(())
}

fn push_first_bookmark(trunk: &str) -> Result<()> {
    let revset = format!("({trunk}..@) & bookmarks()");
    let output = cmd!(
        "jj",
        "log",
        "-r",
        &revset,
        "--reversed",
        "--no-graph",
        "-T",
        r#"bookmarks ++ "\n""#,
        "--limit",
        "1"
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .wrap_err("failed to get bookmarks")?;

    if let Some(bookmark) = output.lines().find(|line| !line.is_empty()) {
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

    Ok(())
}
