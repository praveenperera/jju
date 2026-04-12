//! JJ command execution helpers for jj_tui

pub mod bookmark;
pub mod diff;
pub mod git;
pub mod rebase;
pub mod revision;
pub mod stack_sync;

/// Get the current operation ID for potential undo
pub fn get_current_op_id() -> eyre::Result<String> {
    jju_jj::ops::OperationOps.current_op_id()
}

/// Restore to a previous operation (undo)
pub fn restore_op(op_id: &str) -> eyre::Result<()> {
    jju_jj::ops::OperationOps.restore(op_id)
}

/// Check if working copy has conflicts
pub fn has_conflicts() -> eyre::Result<bool> {
    jju_jj::ops::ConflictOps.has_conflicts()
}

/// Check if rev1 is an ancestor of rev2
pub fn is_ancestor(rev1: &str, rev2: &str) -> eyre::Result<bool> {
    jju_jj::ops::is_ancestor(rev1, rev2)
}

/// List files with conflicts in the working copy
pub fn list_conflict_files() -> eyre::Result<Vec<String>> {
    jju_jj::ops::ConflictOps.list_conflict_files()
}
