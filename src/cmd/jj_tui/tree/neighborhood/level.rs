use super::super::{TreeState, ViewMode};

impl TreeState {
    pub fn expand_neighborhood(&mut self) -> bool {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return false;
        };
        if !state.expand() {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        true
    }

    pub fn shrink_neighborhood(&mut self) -> bool {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return false;
        };
        if !state.shrink() {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        true
    }
}
