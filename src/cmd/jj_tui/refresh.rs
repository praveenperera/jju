//! Shared tree refresh helpers for jj_tui

use super::state::DiffStats;
use super::tree::TreeState;
use crate::jj_lib_helpers::JjRepo;

/// Reload the tree while preserving cursor and focus state when possible
pub fn refresh_tree(
    tree: &mut TreeState,
    diff_stats_cache: &mut std::collections::HashMap<String, DiffStats>,
) -> eyre::Result<()> {
    let current_change_id = tree.current_node().map(|n| n.change_id.clone());
    let parent_change_id = tree
        .current_node()
        .and_then(|n| n.parent_ids.first().cloned());
    let old_cursor = tree.cursor;

    let focus_stack_change_ids: Vec<String> = tree
        .focus_stack
        .iter()
        .filter_map(|&idx| tree.nodes.get(idx).map(|n| n.change_id.clone()))
        .collect();

    let jj_repo = JjRepo::load(None)?;
    *tree = TreeState::load(&jj_repo)?;
    tree.clear_selection();
    diff_stats_cache.clear();

    for change_id in focus_stack_change_ids {
        if let Some(node_idx) = tree.nodes.iter().position(|n| n.change_id == change_id) {
            tree.focus_on(node_idx);
        }
    }

    let find_visible = |cid: &str| {
        tree.visible_entries
            .iter()
            .position(|e| tree.nodes[e.node_index].change_id == cid)
    };

    if let Some(ref cid) = current_change_id
        && let Some(idx) = find_visible(cid)
    {
        tree.cursor = idx;
    } else if let Some(ref pid) = parent_change_id
        && let Some(idx) = find_visible(pid)
    {
        tree.cursor = idx;
    } else {
        tree.cursor = old_cursor.min(tree.visible_count().saturating_sub(1));
    }

    Ok(())
}
