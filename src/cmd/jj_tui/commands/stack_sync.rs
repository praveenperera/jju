/// Detect the trunk branch name via jj's trunk() revset
pub fn detect_trunk_branch() -> eyre::Result<String> {
    jju_jj::stack_sync::detect_trunk_branch()
}

/// Sync trunk bookmark to match its remote tracking branch
pub fn sync_trunk_bookmark(trunk: &str) -> eyre::Result<()> {
    jju_jj::stack_sync::sync_trunk_bookmark(trunk)
}

/// Find the root commits between trunk and the working copy
pub fn find_stack_roots(trunk: &str) -> eyre::Result<Vec<String>> {
    jju_jj::stack_sync::find_stack_roots(trunk)
}

/// Get the first line of a commit's description
pub fn get_commit_description(rev: &str) -> eyre::Result<String> {
    jju_jj::stack_sync::get_commit_description(rev)
}

/// Rebase a stack root onto trunk with --skip-emptied
pub fn rebase_root_onto_trunk(root: &str, trunk: &str) -> eyre::Result<()> {
    jju_jj::stack_sync::rebase_root_onto_trunk(root, trunk)
}

/// Delete tracked bookmarks that are marked as [deleted] on the remote
pub fn cleanup_deleted_bookmarks() -> eyre::Result<Vec<String>> {
    jju_jj::stack_sync::cleanup_deleted_bookmarks()
}
