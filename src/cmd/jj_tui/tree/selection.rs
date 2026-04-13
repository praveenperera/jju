use super::TreeState;

impl TreeState {
    pub fn toggle_selected(&mut self, idx: usize) {
        if self.view.selected.contains(&idx) {
            self.view.selected.remove(&idx);
        } else {
            self.view.selected.insert(idx);
        }
    }

    pub fn select_range(&mut self, from: usize, to: usize) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };
        for index in start..=end {
            self.view.selected.insert(index);
        }
    }

    pub fn clear_selection(&mut self) {
        self.view.selected.clear();
        self.view.selection_anchor = None;
    }
}
