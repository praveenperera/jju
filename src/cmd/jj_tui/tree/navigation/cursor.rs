use super::super::TreeState;

impl TreeState {
    pub fn move_cursor_up(&mut self) {
        if self.view.cursor > 0 {
            self.view.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.view.cursor + 1 < self.visible_count() {
            self.view.cursor += 1;
        }
    }

    pub fn move_cursor_top(&mut self) {
        self.view.cursor = 0;
        self.view.scroll_offset = 0;
    }

    pub fn move_cursor_bottom(&mut self) {
        let count = self.visible_count();
        if count > 0 {
            self.view.cursor = count - 1;
        }
    }

    pub fn jump_to_working_copy(&mut self) {
        for (index, entry) in self.projection.visible_entries.iter().enumerate() {
            if self.snapshot.nodes[entry.node_index].is_working_copy {
                self.view.cursor = index;
                return;
            }
        }
    }
}
