use super::{TreeNode, TreeState};
use ahash::HashMap;

impl TreeState {
    pub fn move_cursor_up(&mut self) {
        if self.view.cursor > 0 {
            self.view.cursor -= 1;
        }
        self.sync_neighborhood_to_cursor();
    }

    pub fn move_cursor_down(&mut self) {
        if self.view.cursor + 1 < self.visible_count() {
            self.view.cursor += 1;
        }
        self.sync_neighborhood_to_cursor();
    }

    pub fn move_cursor_top(&mut self) {
        self.view.cursor = 0;
        self.view.scroll_offset = 0;
        self.sync_neighborhood_to_cursor();
    }

    pub fn move_cursor_bottom(&mut self) {
        let count = self.visible_count();
        if count > 0 {
            self.view.cursor = count - 1;
        }
        self.sync_neighborhood_to_cursor();
    }

    pub fn jump_to_working_copy(&mut self) {
        for (index, entry) in self.projection.visible_entries.iter().enumerate() {
            if self.snapshot.nodes[entry.node_index].is_working_copy {
                self.view.cursor = index;
                self.sync_neighborhood_to_cursor();
                return;
            }
        }
    }

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
        self.sync_neighborhood_to_cursor();
    }

    pub fn page_down(&mut self, amount: usize) {
        let max = self.visible_count().saturating_sub(1);
        self.view.cursor = (self.view.cursor + amount).min(max);
        self.sync_neighborhood_to_cursor();
    }

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

    pub fn bookmark_to_visible_index(&self) -> HashMap<String, usize> {
        let mut map = HashMap::default();
        for (visible_idx, entry) in self.projection.visible_entries.iter().enumerate() {
            let node = &self.snapshot.nodes[entry.node_index];
            for bookmark in &node.bookmarks {
                map.insert(bookmark.name.clone(), visible_idx);
            }
        }
        map
    }
}
