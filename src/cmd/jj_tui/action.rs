//! Action types for jj_tui
//!
//! Actions represent user intents - what the user wants to do
//! The controller maps key events to actions, and the engine
//! processes actions to produce state changes and effects

mod routing;

use super::state::{BookmarkSelectAction, RebaseType};

pub use routing::ActionDomain;

/// All possible user actions in the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Navigation
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorTop,
    MoveCursorBottom,
    JumpToWorkingCopy,
    PageUp(usize),
    PageDown(usize),
    CenterCursor(usize),

    // Focus/View
    ToggleFocus,
    ToggleNeighborhood,
    ExpandNeighborhood,
    ShrinkNeighborhood,
    Unfocus,
    ToggleExpanded,
    ToggleFullMode,
    ToggleSplitView,

    // Mode transitions
    EnterHelp,
    ExitHelp,
    EnterDiffView,
    ExitDiffView,
    EnterConfirmAbandon,
    EnterConfirmStackSync,
    EnterConfirmRebaseOntoTrunk(RebaseType),
    ConfirmYes,
    ConfirmNo,
    EnterSelecting,
    ExitSelecting,
    EnterRebaseMode(RebaseType),
    ExitRebaseMode,
    EnterSquashMode,
    ExitSquashMode,
    EnterMoveBookmarkMode,
    EnterBookmarkPicker(BookmarkSelectAction),
    ExitBookmarkMode,

    // Selection
    ToggleSelection,
    ClearSelection,

    // Rebase mode navigation
    MoveRebaseDestUp,
    MoveRebaseDestDown,
    ToggleRebaseBranches,
    ExecuteRebase,

    // Squash mode navigation
    MoveSquashDestUp,
    MoveSquashDestDown,
    ExecuteSquash,

    // Bookmark modes navigation
    MoveBookmarkDestUp,
    MoveBookmarkDestDown,
    ExecuteBookmarkMove,
    SelectBookmarkUp,
    SelectBookmarkDown,
    ConfirmBookmarkSelect,
    BookmarkPickerUp,
    BookmarkPickerDown,
    BookmarkFilterChar(char),
    BookmarkFilterBackspace,
    ConfirmBookmarkPicker,

    // Help view scrolling
    ScrollHelpUp(usize),
    ScrollHelpDown(usize),

    // Diff view scrolling
    ScrollDiffUp(usize),
    ScrollDiffDown(usize),
    ScrollDiffTop,
    ScrollDiffBottom,

    // Commands (produce effects)
    EditWorkingCopy,
    CreateNewCommit,
    CommitWorkingCopy,
    EditDescription,
    Undo,
    GitPush,
    GitPushAll,
    GitFetch,
    GitImport,
    GitExport,
    CreatePR,

    // Push select mode
    PushSelectUp,
    PushSelectDown,
    PushSelectToggle,
    PushSelectAll,
    PushSelectNone,
    PushSelectFilterChar(char),
    PushSelectFilterBackspace,
    PushSelectConfirm,
    ExitPushSelect,

    // Prefix keys
    SetPendingKey(char),
    ClearPendingKey,

    // Conflicts panel
    EnterConflicts,
    ExitConflicts,
    ConflictsUp,
    ConflictsDown,
    ConflictsJump,
    StartResolveFromConflicts,

    // Divergence resolution
    ResolveDivergence,

    // Lifecycle
    Quit,
    RefreshTree,
    Noop,
}
