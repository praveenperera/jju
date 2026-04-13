mod modal;
mod normal;
mod operations;

use crate::cmd::jj_tui::keybindings::display::KeyFormat;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DynamicHintValue {
    RebaseBranches,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HintSpec {
    Command {
        label: &'static str,
        value: &'static str,
    },
    CommandAnyPending {
        label: &'static str,
        format: KeyFormat,
        value: &'static str,
    },
    CommandPair {
        left: &'static str,
        right: &'static str,
        value: &'static str,
    },
    CommandPairAnyPending {
        left: &'static str,
        right: &'static str,
        format: KeyFormat,
        value: &'static str,
    },
    LabelKeys {
        label: &'static str,
        value: &'static str,
    },
    Literal(&'static str),
    DynamicCommand {
        label: &'static str,
        value: DynamicHintValue,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HintScenario {
    NormalSelection,
    NormalNeighborhood,
    NormalFocus,
    NormalBookmarked,
    NormalDefault,
    Help,
    Diff,
    Confirm,
    Selecting,
    Rebase,
    Squash,
    MovingBookmark,
    BookmarkSelect,
    BookmarkPicker,
    PushSelect,
    Conflicts,
}

pub(crate) fn hint_specs(scenario: HintScenario) -> &'static [HintSpec] {
    match scenario {
        HintScenario::NormalSelection => normal::NORMAL_SELECTION_HINTS,
        HintScenario::NormalNeighborhood => normal::NORMAL_NEIGHBORHOOD_HINTS,
        HintScenario::NormalFocus => normal::NORMAL_FOCUS_HINTS,
        HintScenario::NormalBookmarked => normal::NORMAL_BOOKMARKED_HINTS,
        HintScenario::NormalDefault => normal::NORMAL_DEFAULT_HINTS,
        HintScenario::Help => modal::HELP_HINTS,
        HintScenario::Diff => modal::DIFF_HINTS,
        HintScenario::Confirm => modal::CONFIRM_HINTS,
        HintScenario::Selecting => modal::SELECTING_HINTS,
        HintScenario::Rebase => operations::REBASE_HINTS,
        HintScenario::Squash => operations::SQUASH_HINTS,
        HintScenario::MovingBookmark => operations::MOVING_BOOKMARK_HINTS,
        HintScenario::BookmarkSelect => operations::BOOKMARK_SELECT_HINTS,
        HintScenario::BookmarkPicker => operations::BOOKMARK_PICKER_HINTS,
        HintScenario::PushSelect => operations::PUSH_SELECT_HINTS,
        HintScenario::Conflicts => operations::CONFLICTS_HINTS,
    }
}
