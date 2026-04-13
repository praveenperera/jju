use crate::cmd::jj_tui::state::ModeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModeId {
    Normal,
    Help,
    Diff,
    Confirm,
    Selecting,
    Rebase,
    Squash,
    MovingBookmark,
    BookmarkSelect,
    BookmarkPicker,
    ClipboardBranchSelect,
    PushSelect,
    Conflicts,
}

pub fn mode_id_from_state(mode: &ModeState) -> ModeId {
    match mode {
        ModeState::Normal => ModeId::Normal,
        ModeState::Help(..) => ModeId::Help,
        ModeState::ViewingDiff(_) => ModeId::Diff,
        ModeState::Confirming(_) => ModeId::Confirm,
        ModeState::Selecting => ModeId::Selecting,
        ModeState::Rebasing(_) => ModeId::Rebase,
        ModeState::Squashing(_) => ModeId::Squash,
        ModeState::MovingBookmark(_) => ModeId::MovingBookmark,
        ModeState::BookmarkSelect(_) => ModeId::BookmarkSelect,
        ModeState::BookmarkPicker(_) => ModeId::BookmarkPicker,
        ModeState::ClipboardBranchSelect(_) => ModeId::ClipboardBranchSelect,
        ModeState::PushSelect(_) => ModeId::PushSelect,
        ModeState::Conflicts(_) => ModeId::Conflicts,
    }
}
