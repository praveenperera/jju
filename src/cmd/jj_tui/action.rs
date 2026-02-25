//! Action types for jj_tui
//!
//! Actions represent user intents - what the user wants to do.
//! The controller maps key events to actions, and the engine
//! processes actions to produce state changes and effects.

use super::state::{BookmarkSelectAction, RebaseType};

/// All possible user actions in the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
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
    EnterConfirmMoveBookmarkBackwards {
        bookmark_name: String,
        dest_rev: String,
        op_before: String,
    },
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
    ExtendSelectionToCursor,
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
    ExecuteAbandon {
        revs: Vec<String>,
    },
    ExecuteRebaseOntoTrunk(RebaseType),
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
