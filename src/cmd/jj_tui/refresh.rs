//! Shared tree refresh helpers for jj_tui

mod remap;

use super::state::DiffStats;
use super::tree::TreeState;
use crate::jj_lib_helpers::JjRepo;
use remap::TreeRefreshRemapper;

/// Reload the tree while preserving cursor and focus state when possible
pub fn refresh_tree(
    tree: &mut TreeState,
    diff_stats_cache: &mut std::collections::HashMap<String, DiffStats>,
) -> eyre::Result<()> {
    let remapper = TreeRefreshRemapper::capture(tree);
    let jj_repo = JjRepo::load(None)?;
    let mut refreshed_tree =
        TreeState::load_with_scope(&jj_repo, "trunk()", remapper.load_scope())?;

    remapper.restore(&mut refreshed_tree);
    refreshed_tree.clear_selection();
    diff_stats_cache.clear();
    *tree = refreshed_tree;

    Ok(())
}
