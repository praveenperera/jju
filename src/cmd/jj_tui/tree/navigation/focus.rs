use super::super::{TreeNode, TreeState};

impl TreeState {
    pub fn toggle_focus(&mut self) {
        if self.is_neighborhood_mode() {
            return;
        }

        let Some(entry) = self.current_entry() else {
            return;
        };
        let current_node_idx = entry.node_index;

        if self.view.focus_stack.last() == Some(&current_node_idx) && self.view.cursor == 0 {
            self.unfocus();
            return;
        }

        self.focus_on(current_node_idx);
    }

    pub fn focus_on(&mut self, node_index: usize) {
        if self.is_neighborhood_mode() {
            return;
        }

        self.view.focus_stack.push(node_index);
        self.recompute_projection();
        self.view.cursor = 0;
        self.view.scroll_offset = 0;
    }

    pub fn unfocus(&mut self) {
        let popped_change_id = self.view.focus_stack.pop().and_then(|index| {
            self.snapshot
                .nodes
                .get(index)
                .map(|node| node.change_id.clone())
        });

        self.recompute_projection();

        if let Some(change_id) = popped_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn is_focused(&self) -> bool {
        !self.view.focus_stack.is_empty()
    }

    pub fn focus_depth(&self) -> usize {
        self.view.focus_stack.len()
    }

    pub fn focused_node(&self) -> Option<&TreeNode> {
        self.view
            .focus_stack
            .last()
            .and_then(|&index| self.snapshot.nodes.get(index))
    }
}
