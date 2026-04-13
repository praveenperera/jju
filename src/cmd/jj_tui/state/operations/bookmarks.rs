#[derive(Debug, Clone)]
pub struct MovingBookmarkState {
    pub bookmark_name: String,
    pub dest_cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkSelectAction {
    Move,
    Delete,
    CreatePR,
}

#[derive(Debug, Clone)]
pub struct BookmarkSelectState {
    pub bookmarks: Vec<String>,
    pub selected_index: usize,
    pub target_rev: String,
    pub action: BookmarkSelectAction,
}

/// State for picking a bookmark from all bookmarks with type-to-filter
#[derive(Debug, Clone)]
pub struct BookmarkPickerState {
    pub all_bookmarks: Vec<String>,
    pub filter: String,
    pub filter_cursor: usize,
    pub selected_index: usize,
    pub target_rev: String,
    pub action: BookmarkSelectAction,
}

impl BookmarkPickerState {
    /// Get bookmarks that match the current filter
    pub fn filtered_bookmarks(&self) -> Vec<&String> {
        if self.filter.is_empty() {
            self.all_bookmarks.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.all_bookmarks
                .iter()
                .filter(|bookmark| bookmark.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }
}
