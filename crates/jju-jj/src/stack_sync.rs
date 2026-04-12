use crate::ops::{BookmarkOps, GitOps};
use colored::Colorize;
use duct::cmd;
use eyre::{Context as _, Result};
use jju_core::stack_sync::{StackRootPlan, StackSyncPlan};
use log::debug;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub struct StackSyncCommand {
    push: bool,
    force: bool,
}

impl StackSyncCommand {
    pub fn new(push: bool, force: bool) -> Self {
        Self { push, force }
    }

    pub fn run(self) -> Result<()> {
        println!("{}", "Fetching from remote...".dimmed());
        GitOps.fetch().wrap_err("failed to fetch")?;

        let plan = discover_plan(self.push)?;
        println!("{}{}", "Syncing ".dimmed(), plan.trunk);
        sync_trunk_bookmark(&plan.trunk)?;

        if plan.is_empty() {
            println!(
                "{}{}{}",
                "No commits after ".dimmed(),
                plan.trunk,
                ", nothing to rebase".dimmed()
            );
            return Ok(());
        }

        if !self.force && !confirm_plan(&plan)? {
            println!("{}", "Aborted".yellow());
            return Ok(());
        }

        execute_plan(&plan)?;
        let _ = cleanup_deleted_bookmarks()?;

        if plan.push_bookmark_after_sync {
            push_first_bookmark(&plan.trunk)?;
        }

        println!("{}", "Stack sync complete".green());
        Ok(())
    }
}

pub fn discover_plan(push_bookmark_after_sync: bool) -> Result<StackSyncPlan> {
    let trunk = detect_trunk_branch()?;
    let roots = find_stack_roots(&trunk)?
        .into_iter()
        .map(|change_id| {
            let description = get_commit_description(&change_id).unwrap_or_default();
            StackRootPlan {
                change_id,
                description,
            }
        })
        .collect();

    Ok(StackSyncPlan {
        trunk,
        roots,
        push_bookmark_after_sync,
    })
}

pub fn execute_plan(plan: &StackSyncPlan) -> Result<()> {
    for root in &plan.roots {
        println!("{}{}...", "Rebasing stack from ".dimmed(), root.change_id);
        cmd!(
            "jj",
            "rebase",
            "--source",
            &root.change_id,
            "--onto",
            &plan.trunk,
            "--skip-emptied"
        )
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err_with(|| format!("failed to rebase from {}", root.change_id))?;
    }

    Ok(())
}

pub fn detect_trunk_branch() -> Result<String> {
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

pub fn find_stack_roots(trunk: &str) -> Result<Vec<String>> {
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
    debug!("stack roots: {roots:?}");
    Ok(roots)
}

pub fn get_commit_description(rev: &str) -> Result<String> {
    cmd!(
        "jj",
        "log",
        "-r",
        rev,
        "--no-graph",
        "-T",
        "description.first_line()"
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .map(|output| output.trim().to_string())
    .wrap_err_with(|| format!("failed to load description for {rev}"))
}

fn confirm_plan(plan: &StackSyncPlan) -> Result<bool> {
    println!(
        "Will rebase the following commits on top of {}:",
        plan.trunk.cyan()
    );

    for root in &plan.roots {
        println!(
            "  {}  {}",
            root.change_id.purple(),
            root.description.dimmed()
        );
        println!(
            "  {}",
            format!(
                "jj rebase --source (-s) {} --onto (-o) {} --skip-emptied",
                root.change_id, plan.trunk
            )
            .dimmed()
        );
    }

    print!("Continue? [y/N] ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

pub fn sync_trunk_bookmark(trunk: &str) -> Result<()> {
    BookmarkOps
        .set(trunk, &format!("{trunk}@origin"))
        .wrap_err("failed to sync trunk bookmark")
}

pub fn rebase_root_onto_trunk(root: &str, trunk: &str) -> Result<()> {
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
    Ok(())
}

pub fn cleanup_deleted_bookmarks() -> Result<Vec<String>> {
    let tracked = cmd!("jj", "bookmark", "list", "--tracked")
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to list tracked bookmarks")?;

    let mut deleted = Vec::new();
    for line in tracked.lines() {
        if line.contains("[deleted]")
            && let Some(bookmark) = line.split_whitespace().next()
        {
            println!("{}{}", "Deleting merged bookmark: ".dimmed(), bookmark);
            BookmarkOps
                .delete(bookmark)
                .wrap_err_with(|| format!("failed to delete bookmark {bookmark}"))?;
            deleted.push(bookmark.to_string());
        }
    }

    Ok(deleted)
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
        GitOps.push_bookmark(bookmark).wrap_err("failed to push")?;
    } else {
        println!("{}", "No bookmarks found to push".dimmed());
    }

    Ok(())
}
