use super::{DynamicHintValue, HintSpec};
use crate::cmd::jj_tui::keybindings::catalog::command_id as cmd;

pub(super) const REBASE_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::DEST_DOWN,
        right: cmd::DEST_UP,
        value: "dest",
    },
    HintSpec::DynamicCommand {
        label: cmd::BRANCHES,
        value: DynamicHintValue::RebaseBranches,
    },
    HintSpec::Command {
        label: cmd::RUN,
        value: "run",
    },
    HintSpec::Command {
        label: cmd::CANCEL,
        value: "cancel",
    },
];

pub(super) const SQUASH_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::DEST_DOWN,
        right: cmd::DEST_UP,
        value: "dest",
    },
    HintSpec::Command {
        label: cmd::RUN,
        value: "run",
    },
    HintSpec::Command {
        label: cmd::CANCEL,
        value: "cancel",
    },
];

pub(super) const MOVING_BOOKMARK_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::DEST_DOWN,
        right: cmd::DEST_UP,
        value: "dest",
    },
    HintSpec::Command {
        label: cmd::RUN,
        value: "run",
    },
    HintSpec::Command {
        label: cmd::CANCEL,
        value: "cancel",
    },
];

pub(super) const BOOKMARK_SELECT_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::DOWN,
        right: cmd::UP,
        value: "navigate",
    },
    HintSpec::Command {
        label: cmd::SELECT,
        value: "select",
    },
    HintSpec::Command {
        label: cmd::CANCEL,
        value: "cancel",
    },
];

pub(super) const BOOKMARK_PICKER_HINTS: &[HintSpec] = &[
    HintSpec::Literal("type:filter"),
    HintSpec::CommandPair {
        left: cmd::UP,
        right: cmd::DOWN,
        value: "navigate",
    },
    HintSpec::Command {
        label: cmd::CONFIRM,
        value: "select",
    },
    HintSpec::Command {
        label: cmd::CANCEL,
        value: "cancel",
    },
];

pub(super) const PUSH_SELECT_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::UP,
        right: cmd::DOWN,
        value: "navigate",
    },
    HintSpec::Command {
        label: cmd::TOGGLE,
        value: "toggle",
    },
    HintSpec::Command {
        label: cmd::ALL,
        value: "all",
    },
    HintSpec::Command {
        label: cmd::NONE,
        value: "none",
    },
    HintSpec::Command {
        label: cmd::PUSH,
        value: "push",
    },
    HintSpec::Command {
        label: cmd::CANCEL,
        value: "cancel",
    },
];

pub(super) const CONFLICTS_HINTS: &[HintSpec] = &[
    HintSpec::CommandPair {
        left: cmd::DOWN,
        right: cmd::UP,
        value: "nav",
    },
    HintSpec::Command {
        label: cmd::RESOLVE,
        value: "resolve",
    },
    HintSpec::LabelKeys {
        label: cmd::EXIT,
        value: "exit",
    },
];
