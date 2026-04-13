use super::HintSpec;
use crate::cmd::jj_tui::keybindings::{catalog::command_id as cmd, display::KeyFormat};

pub(super) const NORMAL_SELECTION_HINTS: &[HintSpec] = &[
    HintSpec::Command {
        label: cmd::ABANDON,
        value: "abandon",
    },
    HintSpec::Command {
        label: cmd::TOGGLE,
        value: "toggle",
    },
    HintSpec::Command {
        label: cmd::ESC,
        value: "clear",
    },
];

pub(super) const NORMAL_NEIGHBORHOOD_HINTS: &[HintSpec] = &[
    HintSpec::CommandAnyPending {
        label: cmd::NEIGHBORHOOD,
        format: KeyFormat::Concat,
        value: "full",
    },
    HintSpec::CommandPairAnyPending {
        left: cmd::NEIGHBORHOOD_MORE,
        right: cmd::NEIGHBORHOOD_LESS,
        format: KeyFormat::Concat,
        value: "size",
    },
    HintSpec::Command {
        label: cmd::ZOOM,
        value: "open",
    },
    HintSpec::Command {
        label: cmd::ESC,
        value: "back",
    },
    HintSpec::Command {
        label: cmd::DIFF,
        value: "diff",
    },
    HintSpec::Command {
        label: cmd::DESC,
        value: "desc",
    },
    HintSpec::Command {
        label: cmd::HELP,
        value: "help",
    },
    HintSpec::Command {
        label: cmd::QUIT,
        value: "quit",
    },
];

pub(super) const NORMAL_FOCUS_HINTS: &[HintSpec] = &[
    HintSpec::Command {
        label: cmd::ZOOM,
        value: "unfocus",
    },
    HintSpec::Command {
        label: cmd::FULL,
        value: "toggle-full",
    },
    HintSpec::Command {
        label: cmd::HELP,
        value: "help",
    },
    HintSpec::Command {
        label: cmd::QUIT,
        value: "quit",
    },
];

pub(super) const NORMAL_BOOKMARKED_HINTS: &[HintSpec] = &[
    HintSpec::Command {
        label: cmd::PUSH,
        value: "push",
    },
    HintSpec::Command {
        label: cmd::STACK_SYNC,
        value: "sync",
    },
    HintSpec::Command {
        label: cmd::BOOKMARK,
        value: "bookmark",
    },
    HintSpec::Command {
        label: cmd::REBASE_SINGLE,
        value: "rebase",
    },
    HintSpec::Command {
        label: cmd::HELP,
        value: "help",
    },
    HintSpec::Command {
        label: cmd::QUIT,
        value: "quit",
    },
];

pub(super) const NORMAL_DEFAULT_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::REBASE_SINGLE,
        right: cmd::REBASE_DESC,
        value: "rebase",
    },
    HintSpec::Command {
        label: cmd::TRUNK_SINGLE,
        value: "trunk",
    },
    HintSpec::Command {
        label: cmd::DESC,
        value: "desc",
    },
    HintSpec::Command {
        label: cmd::STACK_SYNC,
        value: "sync",
    },
    HintSpec::Command {
        label: cmd::BOOKMARK,
        value: "bookmark",
    },
    HintSpec::Command {
        label: cmd::GIT,
        value: "git",
    },
    HintSpec::Command {
        label: cmd::NAV,
        value: "nav",
    },
    HintSpec::Command {
        label: cmd::HELP,
        value: "help",
    },
    HintSpec::Command {
        label: cmd::QUIT,
        value: "quit",
    },
];
