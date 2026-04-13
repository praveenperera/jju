#[derive(Debug, Clone)]
pub struct PushSelectState {
    pub all_bookmarks: Vec<String>,
    pub filter: String,
    pub filter_cursor: usize,
    pub cursor_index: usize,
    pub selected: ahash::HashSet<usize>, // indices into all_bookmarks (not filtered)
}

impl PushSelectState {
    /// Get bookmarks that match the current filter with their original indices
    pub fn filtered_bookmarks(&self) -> Vec<(usize, &str)> {
        if self.filter.is_empty() {
            self.all_bookmarks
                .iter()
                .enumerate()
                .map(|(index, bookmark)| (index, bookmark.as_str()))
                .collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.all_bookmarks
                .iter()
                .enumerate()
                .filter(|(_, bookmark)| bookmark.to_lowercase().contains(&filter_lower))
                .map(|(index, bookmark)| (index, bookmark.as_str()))
                .collect()
        }
    }

    /// Count selected bookmarks in the filtered view
    pub fn selected_filtered_count(&self) -> usize {
        self.filtered_bookmarks()
            .iter()
            .filter(|(index, _)| self.selected.contains(index))
            .count()
    }
}
