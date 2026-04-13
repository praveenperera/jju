use super::{
    BookmarkPickerState, BookmarkSelectState, ConfirmState, ConflictsState, DiffState,
    MovingBookmarkState, PushSelectState, RebaseState, SquashState,
};

/// Unified mode state - single source of truth for current mode and its associated state
#[derive(Debug, Clone)]
pub enum ModeState {
    Normal,
    Help(HelpState),
    ViewingDiff(DiffState),
    Confirming(ConfirmState),
    Selecting,
    Rebasing(RebaseState),
    MovingBookmark(MovingBookmarkState),
    BookmarkSelect(BookmarkSelectState),
    BookmarkPicker(BookmarkPickerState),
    PushSelect(PushSelectState),
    Squashing(SquashState),
    Conflicts(ConflictsState),
}

#[derive(Debug, Clone)]
pub struct HelpState {
    pub scroll_offset: usize,
}
