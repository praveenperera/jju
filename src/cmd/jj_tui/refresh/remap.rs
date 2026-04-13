mod capture;
mod restore;
#[cfg(test)]
mod tests;

use crate::cmd::jj_tui::tree::{TreeLoadScope, TreeState, ViewMode};

#[derive(Debug, Clone)]
pub(super) struct TreeRefreshRemapper {
    current_change_id: Option<String>,
    parent_change_id: Option<String>,
    old_cursor: usize,
    full_mode: bool,
    load_scope: TreeLoadScope,
    view_mode: ViewMode,
    focus_stack_change_ids: Vec<String>,
}

impl TreeRefreshRemapper {
    pub(in crate::cmd::jj_tui::refresh) fn capture(tree: &TreeState) -> Self {
        capture::capture(tree)
    }

    pub(in crate::cmd::jj_tui::refresh) fn restore(&self, tree: &mut TreeState) {
        restore::restore(self, tree)
    }

    pub(super) fn load_scope(&self) -> TreeLoadScope {
        self.load_scope
    }
}

fn find_node_index(tree: &TreeState, change_id: &str) -> Option<usize> {
    tree.nodes()
        .iter()
        .position(|node| node.change_id == change_id)
}

fn find_visible_index(tree: &TreeState, change_id: &str) -> Option<usize> {
    tree.visible_entries()
        .iter()
        .position(|entry| tree.nodes()[entry.node_index].change_id == change_id)
}
