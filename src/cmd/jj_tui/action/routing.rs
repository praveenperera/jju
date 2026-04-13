use super::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionDomain {
    Navigation,
    Modes,
    Bookmarks,
    Commands,
    Lifecycle,
}

impl Action {
    pub fn domain(&self) -> ActionDomain {
        match self {
            Action::MoveCursorUp
            | Action::MoveCursorDown
            | Action::MoveCursorTop
            | Action::MoveCursorBottom
            | Action::JumpToWorkingCopy
            | Action::PageUp(_)
            | Action::PageDown(_)
            | Action::CenterCursor(_)
            | Action::ToggleFocus
            | Action::ToggleNeighborhood
            | Action::ExpandNeighborhood
            | Action::ShrinkNeighborhood
            | Action::EnterNeighborhoodPath
            | Action::ExitNeighborhoodPath
            | Action::Unfocus
            | Action::ToggleExpanded
            | Action::ToggleFullMode
            | Action::ToggleSplitView
            | Action::EnterHelp
            | Action::ExitHelp
            | Action::ScrollHelpUp(_)
            | Action::ScrollHelpDown(_)
            | Action::EnterSelecting
            | Action::ExitSelecting
            | Action::ToggleSelection
            | Action::ClearSelection
            | Action::RefreshTree => ActionDomain::Navigation,
            Action::EnterDiffView
            | Action::ExitDiffView
            | Action::EnterConfirmAbandon
            | Action::EnterConfirmStackSync
            | Action::EnterConfirmRebaseOntoTrunk(_)
            | Action::ConfirmYes
            | Action::ConfirmNo
            | Action::EnterRebaseMode(_)
            | Action::ExitRebaseMode
            | Action::MoveRebaseDestUp
            | Action::MoveRebaseDestDown
            | Action::ToggleRebaseBranches
            | Action::ExecuteRebase
            | Action::EnterSquashMode
            | Action::ExitSquashMode
            | Action::MoveSquashDestUp
            | Action::MoveSquashDestDown
            | Action::ExecuteSquash
            | Action::ScrollDiffUp(_)
            | Action::ScrollDiffDown(_)
            | Action::ScrollDiffTop
            | Action::ScrollDiffBottom
            | Action::EnterConflicts
            | Action::ExitConflicts
            | Action::ConflictsUp
            | Action::ConflictsDown
            | Action::ConflictsJump
            | Action::StartResolveFromConflicts => ActionDomain::Modes,
            Action::EnterMoveBookmarkMode
            | Action::EnterBookmarkPicker(_)
            | Action::ExitBookmarkMode
            | Action::MoveBookmarkDestUp
            | Action::MoveBookmarkDestDown
            | Action::ExecuteBookmarkMove
            | Action::SelectBookmarkUp
            | Action::SelectBookmarkDown
            | Action::ConfirmBookmarkSelect
            | Action::BookmarkPickerUp
            | Action::BookmarkPickerDown
            | Action::BookmarkFilterChar(_)
            | Action::BookmarkFilterBackspace
            | Action::ConfirmBookmarkPicker
            | Action::GitPush
            | Action::GitPushAll
            | Action::PushSelectUp
            | Action::PushSelectDown
            | Action::PushSelectToggle
            | Action::PushSelectAll
            | Action::PushSelectNone
            | Action::PushSelectFilterChar(_)
            | Action::PushSelectFilterBackspace
            | Action::PushSelectConfirm
            | Action::ExitPushSelect => ActionDomain::Bookmarks,
            Action::EditWorkingCopy
            | Action::CreateNewCommit
            | Action::CommitWorkingCopy
            | Action::EditDescription
            | Action::Undo
            | Action::GitFetch
            | Action::GitImport
            | Action::GitExport
            | Action::ResolveDivergence
            | Action::CreatePR => ActionDomain::Commands,
            Action::SetPendingKey(_) | Action::ClearPendingKey | Action::Quit | Action::Noop => {
                ActionDomain::Lifecycle
            }
        }
    }

    pub fn clears_pending_key(&self) -> bool {
        !matches!(self, Self::SetPendingKey(_))
    }
}
