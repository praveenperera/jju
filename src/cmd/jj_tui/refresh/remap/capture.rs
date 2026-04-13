use super::TreeRefreshRemapper;
use crate::cmd::jj_tui::tree::TreeState;

pub(super) fn capture(tree: &TreeState) -> TreeRefreshRemapper {
    TreeRefreshRemapper {
        current_change_id: tree.current_node().map(|node| node.change_id.clone()),
        parent_change_id: tree
            .current_node()
            .and_then(|node| node.parent_ids.first().cloned()),
        old_cursor: tree.view.cursor,
        full_mode: tree.view.full_mode,
        load_scope: tree.view.load_scope,
        view_mode: tree.view.view_mode.clone(),
        focus_stack_change_ids: tree
            .view
            .focus_stack
            .iter()
            .filter_map(|&index| tree.nodes().get(index).map(|node| node.change_id.clone()))
            .collect(),
    }
}
