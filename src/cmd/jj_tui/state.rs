//! State types for jj_tui
//!
//! This module contains all the state enums and structs used by the TUI

mod diff;
mod message;
mod mode;
mod operations;

pub use diff::{DiffLine, DiffLineKind, DiffState, DiffStats, StyledSpan};
pub use message::{MessageKind, StatusMessage};
pub use mode::{HelpState, ModeState};
pub use operations::{
    BookmarkPickerState, BookmarkSelectAction, BookmarkSelectState, ClipboardBranchOption,
    ClipboardBranchSelectState, ConfirmAction, ConfirmState, ConflictsState, MovingBookmarkState,
    PushSelectState, RebaseState, RebaseType, SquashState,
};
