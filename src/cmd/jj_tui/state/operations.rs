mod bookmarks;
mod clipboard;
mod confirm;
mod push_select;
mod rebase;

pub use bookmarks::{
    BookmarkPickerState, BookmarkSelectAction, BookmarkSelectState, MovingBookmarkState,
};
pub use clipboard::{ClipboardBranchOption, ClipboardBranchSelectState};
pub use confirm::{ConfirmAction, ConfirmState};
pub use push_select::PushSelectState;
pub use rebase::{RebaseState, RebaseType};

#[derive(Debug, Clone, Default)]
pub struct ConflictsState {
    pub files: Vec<String>,
    pub selected_index: usize,
}

#[derive(Debug, Clone)]
pub struct SquashState {
    pub source_revs: Vec<String>,
    pub dest_cursor: usize,
    pub op_before: String,
}
