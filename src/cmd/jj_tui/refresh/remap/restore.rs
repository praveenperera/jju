use super::{TreeRefreshRemapper, find_node_index, find_visible_index};
use crate::cmd::jj_tui::tree::{NeighborhoodState, TreeState, ViewMode};

pub(super) fn restore(remapper: &TreeRefreshRemapper, tree: &mut TreeState) {
    tree.view.full_mode = remapper.full_mode;
    remapper.restore_mode(tree);
    remapper.restore_cursor(tree);
}

impl TreeRefreshRemapper {
    fn restore_mode(&self, tree: &mut TreeState) {
        match &self.view_mode {
            ViewMode::Tree => self.restore_focus_stack(tree),
            ViewMode::Neighborhood(state) => {
                tree.set_view_mode(ViewMode::Neighborhood(NeighborhoodState {
                    anchor_change_id: restored_anchor_change_id(
                        tree,
                        state,
                        self.current_change_id.as_deref(),
                    ),
                    history: restored_history(tree, &state.history),
                    extent: state.extent.clone(),
                }))
            }
        }
    }

    fn restore_focus_stack(&self, tree: &mut TreeState) {
        for change_id in &self.focus_stack_change_ids {
            let Some(node_index) = find_node_index(tree, change_id) else {
                continue;
            };
            tree.focus_on(node_index);
        }
    }

    fn restore_cursor(&self, tree: &mut TreeState) {
        if let Some(change_id) = &self.current_change_id
            && let Some(index) = find_visible_index(tree, change_id)
        {
            tree.view.cursor = index;
            return;
        }

        if let Some(change_id) = &self.parent_change_id
            && let Some(index) = find_visible_index(tree, change_id)
        {
            tree.view.cursor = index;
            return;
        }

        tree.view.cursor = self.old_cursor.min(tree.visible_count().saturating_sub(1));
    }
}

fn restored_anchor_change_id(
    tree: &TreeState,
    state: &NeighborhoodState,
    current_change_id: Option<&str>,
) -> String {
    if find_node_index(tree, &state.anchor_change_id).is_some() {
        state.anchor_change_id.clone()
    } else if let Some(change_id) = current_change_id {
        change_id.to_string()
    } else {
        state.anchor_change_id.clone()
    }
}

fn restored_history(tree: &TreeState, history: &[String]) -> Vec<String> {
    history
        .iter()
        .filter(|change_id| find_node_index(tree, change_id).is_some())
        .cloned()
        .collect()
}
