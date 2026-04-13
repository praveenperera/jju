use crate::cmd::jj_tui::keybindings::{ModeId, catalog::HintScenario};

#[derive(Debug, Clone, Copy)]
pub struct StatusHintContext {
    pub mode: ModeId,
    pub has_selection: bool,
    pub has_focus: bool,
    pub neighborhood_active: bool,
    pub current_has_bookmark: bool,
    pub rebase_allow_branches: Option<bool>,
}

pub(super) fn scenario_for_context(ctx: &StatusHintContext) -> HintScenario {
    match ctx.mode {
        ModeId::Normal if ctx.has_selection => HintScenario::NormalSelection,
        ModeId::Normal if ctx.neighborhood_active => HintScenario::NormalNeighborhood,
        ModeId::Normal if ctx.has_focus => HintScenario::NormalFocus,
        ModeId::Normal if ctx.current_has_bookmark => HintScenario::NormalBookmarked,
        ModeId::Normal => HintScenario::NormalDefault,
        ModeId::Help => HintScenario::Help,
        ModeId::Diff => HintScenario::Diff,
        ModeId::Confirm => HintScenario::Confirm,
        ModeId::Selecting => HintScenario::Selecting,
        ModeId::Rebase => HintScenario::Rebase,
        ModeId::Squash => HintScenario::Squash,
        ModeId::MovingBookmark => HintScenario::MovingBookmark,
        ModeId::BookmarkSelect => HintScenario::BookmarkSelect,
        ModeId::BookmarkPicker => HintScenario::BookmarkPicker,
        ModeId::PushSelect => HintScenario::PushSelect,
        ModeId::Conflicts => HintScenario::Conflicts,
    }
}
