//! JJ command execution helpers for jj_tui

mod common;

pub mod bookmark;
pub mod diff;
pub mod git;
pub mod rebase;
pub mod resolve;
pub mod revision;
pub mod stack_sync;

use duct::cmd;
use eyre::Result;

/// Get the current operation ID for potential undo
pub fn get_current_op_id() -> Result<String> {
    let output = cmd!("jj", "op", "log", "--limit", "1", "-T", "id", "--no-graph")
        .stdout_capture()
        .stderr_null()
        .read()?;
    let op_id = output.trim().to_string();
    if op_id.is_empty() {
        eyre::bail!("jj op log returned empty operation ID");
    }
    Ok(op_id)
}

/// Restore to a previous operation (undo)
pub fn restore_op(op_id: &str) -> Result<()> {
    common::run_with_stderr(cmd!("jj", "op", "restore", op_id))
}

/// Check if working copy has conflicts
pub fn has_conflicts() -> Result<bool> {
    let output = cmd!("jj", "log", "-r", "@", "-T", r#"if(conflict, "conflict")"#)
        .stdout_capture()
        .stderr_null()
        .read()?;
    Ok(output.contains("conflict"))
}

/// Check if rev1 is an ancestor of rev2
pub fn is_ancestor(rev1: &str, rev2: &str) -> Result<bool> {
    let revset = format!("{rev1} & ::({rev2})");
    let output = cmd!(
        "jj",
        "log",
        "-r",
        revset,
        "--no-graph",
        "-T",
        "change_id",
        "--limit",
        "1"
    )
    .stdout_capture()
    .stderr_null()
    .read()?;
    Ok(!output.trim().is_empty())
}

/// List files with conflicts in the working copy
pub fn list_conflict_files() -> Result<Vec<String>> {
    let template = r#"conflict_files.map(|x| x ++ "\n").join("")"#;
    let output = cmd!("jj", "log", "-r", "@", "-T", template, "--no-graph")
        .stdout_capture()
        .stderr_null()
        .read()?;
    let files: Vec<String> = output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(files)
}
