use super::super::{NeighborhoodState, TreeLoadScope, TreeState, ViewMode};

impl TreeState {
    pub fn enable_neighborhood(&mut self) {
        let Some(anchor_change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return;
        };
        self.view.load_scope = TreeLoadScope::Neighborhood;
        self.view.focus_stack.clear();
        self.set_view_mode(ViewMode::Neighborhood(NeighborhoodState::new(
            anchor_change_id.clone(),
        )));
        self.restore_cursor_to_change_id(&anchor_change_id);
    }

    pub fn disable_neighborhood(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        self.view.load_scope = TreeLoadScope::Stack;
        self.set_view_mode(ViewMode::Tree);
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn toggle_neighborhood(&mut self) {
        if self.is_neighborhood_mode() {
            self.disable_neighborhood();
        } else {
            self.enable_neighborhood();
        }
    }
}
