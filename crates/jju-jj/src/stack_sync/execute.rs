use super::{discover_plan, print_aborted, print_complete, should_continue};
use crate::ops::GitOps;
use colored::Colorize;
use duct::cmd;
use eyre::{Context as _, Result};
use jju_core::stack_sync::StackSyncPlan;

pub(super) fn run_command(command: super::StackSyncCommand) -> Result<()> {
    println!("{}", "Fetching from remote...".dimmed());
    GitOps.fetch().wrap_err("failed to fetch")?;

    let plan = discover_plan(command.push)?;
    println!("{}{}", "Syncing ".dimmed(), plan.trunk);
    super::sync_trunk_bookmark(&plan.trunk)?;

    if plan.is_empty() {
        println!(
            "{}{}{}",
            "No commits after ".dimmed(),
            plan.trunk,
            ", nothing to rebase".dimmed()
        );
        return Ok(());
    }

    if !should_continue(&plan, command.force)? {
        print_aborted();
        return Ok(());
    }

    execute_plan(&plan)?;
    let _ = super::cleanup_deleted_bookmarks()?;

    if plan.push_bookmark_after_sync {
        push_first_bookmark(&plan.trunk)?;
    }

    print_complete();
    Ok(())
}

pub fn execute_plan(plan: &StackSyncPlan) -> Result<()> {
    for root in &plan.roots {
        println!("{}{}...", "Rebasing stack from ".dimmed(), root.change_id);
        rebase_root_onto_trunk(&root.change_id, &plan.trunk)
            .wrap_err_with(|| format!("failed to rebase from {}", root.change_id))?;
    }

    Ok(())
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
