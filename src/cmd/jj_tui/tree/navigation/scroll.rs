use super::super::TreeState;

impl TreeState {
    pub fn update_scroll(&mut self, viewport_height: usize, cursor_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.view.cursor < self.view.scroll_offset {
            self.view.scroll_offset = self.view.cursor;
        } else if self.view.cursor + cursor_height > self.view.scroll_offset + viewport_height {
            self.view.scroll_offset =
                (self.view.cursor + cursor_height).saturating_sub(viewport_height);
        }
    }

    pub fn page_up(&mut self, amount: usize) {
        self.view.cursor = self.view.cursor.saturating_sub(amount);
    }

    pub fn page_down(&mut self, amount: usize) {
        let max = self.visible_count().saturating_sub(1);
        self.view.cursor = (self.view.cursor + amount).min(max);
    }
}
