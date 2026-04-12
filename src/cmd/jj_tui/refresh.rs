//! Shared tree refresh helpers for jj_tui

use super::state::DiffStats;
use super::tree::{NeighborhoodAnchor, NeighborhoodState, TreeState, ViewMode};
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
    let old_cursor = tree.view.cursor;
    let old_full_mode = tree.view.full_mode;
    let old_load_scope = tree.view.load_scope;
    let old_view_mode = tree.view.view_mode.clone();

    let focus_stack_change_ids: Vec<String> = tree
        .view
        .focus_stack
        .iter()
        .filter_map(|&idx| tree.nodes().get(idx).map(|node| node.change_id.clone()))
        .collect();

    let jj_repo = JjRepo::load(None)?;
    *tree = TreeState::load_with_scope(&jj_repo, "trunk()", old_load_scope)?;
    tree.view.full_mode = old_full_mode;
    tree.set_view_mode(tree.view.view_mode.clone());
    tree.clear_selection();
    diff_stats_cache.clear();

    if matches!(&old_view_mode, ViewMode::Tree) {
        for change_id in focus_stack_change_ids {
            if let Some(node_idx) = tree
                .nodes()
                .iter()
                .position(|node| node.change_id == change_id)
            {
                tree.focus_on(node_idx);
            }
        }
    } else if let Some((anchor, level)) =
        refresh_anchor(&old_view_mode, current_change_id.as_deref())
    {
        tree.set_view_mode(ViewMode::Neighborhood(NeighborhoodState {
            anchor: NeighborhoodAnchor::Fixed(anchor),
            level,
        }));
    }

    let find_visible = |cid: &str| {
        tree.visible_entries()
            .iter()
            .position(|entry| tree.nodes()[entry.node_index].change_id == cid)
    };

    if let Some(ref cid) = current_change_id
        && let Some(idx) = find_visible(cid)
    {
        tree.view.cursor = idx;
    } else if let Some(ref pid) = parent_change_id
        && let Some(idx) = find_visible(pid)
    {
        tree.view.cursor = idx;
    } else {
        tree.view.cursor = old_cursor.min(tree.visible_count().saturating_sub(1));
    }

    if matches!(
        &old_view_mode,
        ViewMode::Neighborhood(NeighborhoodState {
            anchor: NeighborhoodAnchor::FollowCursor,
            ..
        })
    ) {
        tree.resume_neighborhood_follow_cursor();
    }

    Ok(())
}

fn refresh_anchor(
    view_mode: &ViewMode,
    current_change_id: Option<&str>,
) -> Option<(String, usize)> {
    match view_mode {
        ViewMode::Tree => None,
        ViewMode::Neighborhood(NeighborhoodState {
            anchor: NeighborhoodAnchor::FollowCursor,
            level,
        }) => current_change_id.map(|change_id| (change_id.to_string(), *level)),
        ViewMode::Neighborhood(NeighborhoodState {
            anchor: NeighborhoodAnchor::Fixed(change_id),
            level,
        }) => Some((change_id.clone(), *level)),
    }
}
