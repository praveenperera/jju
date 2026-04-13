use super::super::TreeState;

impl TreeState {
    pub fn toggle_expanded(&mut self) {
        if self.view.expanded_entry == Some(self.view.cursor) {
            self.view.expanded_entry = None;
        } else {
            self.view.expanded_entry = Some(self.view.cursor);
        }
    }

    pub fn is_expanded(&self, visible_idx: usize) -> bool {
        self.view.expanded_entry == Some(visible_idx)
    }
}
