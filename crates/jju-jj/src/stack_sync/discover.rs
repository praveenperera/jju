use crate::ops::BookmarkOps;
use colored::Colorize;
use duct::cmd;
use eyre::{Context as _, Result};
use jju_core::stack_sync::{StackRootPlan, StackSyncPlan};
use log::debug;

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

pub fn sync_trunk_bookmark(trunk: &str) -> Result<()> {
    BookmarkOps
        .set(trunk, &format!("{trunk}@origin"))
        .wrap_err("failed to sync trunk bookmark")
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
