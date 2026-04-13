use super::HintSpec;
use crate::cmd::jj_tui::keybindings::{catalog::command_id as cmd, display::KeyFormat};

pub(super) const HELP_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::SCROLL_DOWN,
        right: cmd::SCROLL_UP,
        value: "scroll",
    },
    HintSpec::LabelKeys {
        label: cmd::CLOSE,
        value: "close",
    },
];

pub(super) const DIFF_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::SCROLL_DOWN,
        right: cmd::SCROLL_UP,
        value: "scroll",
    },
    HintSpec::CommandPair {
        left: cmd::PAGE_DOWN,
        right: cmd::PAGE_UP,
        value: "page",
    },
    HintSpec::CommandPairAnyPending {
        left: cmd::TOP,
        right: cmd::BOTTOM,
        format: KeyFormat::Concat,
        value: "top/bottom",
    },
    HintSpec::LabelKeys {
        label: cmd::CLOSE,
        value: "close",
    },
];

pub(super) const CONFIRM_HINTS: &[HintSpec] = &[
    HintSpec::LabelKeys {
        label: cmd::YES,
        value: "yes",
    },
    HintSpec::LabelKeys {
        label: cmd::NO,
        value: "no",
    },
];

pub(super) const SELECTING_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::DOWN,
        right: cmd::UP,
        value: "extend",
    },
    HintSpec::Command {
        label: cmd::ABANDON,
        value: "abandon",
    },
    HintSpec::Command {
        label: cmd::EXIT,
        value: "exit",
    },
];
