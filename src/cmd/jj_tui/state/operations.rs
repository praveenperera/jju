#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseType {
    Single,          // -r: just this revision
    WithDescendants, // -s: revision + all descendants
}

impl std::fmt::Display for RebaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebaseType::Single => write!(f, "-r"),
            RebaseType::WithDescendants => write!(f, "-s"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Abandon,
    StackSync,
    RebaseOntoTrunk(RebaseType),
    MoveBookmarkBackwards {
        bookmark_name: String,
        dest_rev: String,
    },
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub action: ConfirmAction,
    pub message: String,
    pub revs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RebaseState {
    pub source_rev: String,
    pub rebase_type: RebaseType,
    pub dest_cursor: usize,
    pub allow_branches: bool,
}

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

/// State for multi-select bookmark push
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

#[derive(Debug, Clone)]
pub struct SquashState {
    pub source_rev: String,
    pub dest_cursor: usize,
    pub op_before: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConflictsState {
    pub files: Vec<String>,
    pub selected_index: usize,
}
