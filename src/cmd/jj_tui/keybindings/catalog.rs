use super::CommandSpec;
use super::bindings;

mod hints;

pub mod command_id {
    pub const ABANDON: &str = "abandon";
    pub const BOOKMARK: &str = "bookmark";
    pub const BOTTOM: &str = "bottom";
    pub const DESC: &str = "desc";
    pub const DEST_DOWN: &str = "dest_down";
    pub const DEST_UP: &str = "dest_up";
    pub const DIFF: &str = "diff";
    pub const DOWN: &str = "down";
    pub const ESC: &str = "esc";
    pub const EXIT: &str = "exit";
    pub const FULL: &str = "full";
    pub const GIT: &str = "git";
    pub const HELP: &str = "help";
    pub const NEIGHBORHOOD: &str = "neighborhood";
    pub const NEIGHBORHOOD_LESS: &str = "neighborhood_less";
    pub const NEIGHBORHOOD_MORE: &str = "neighborhood_more";
    pub const PAGE_DOWN: &str = "page_down";
    pub const PAGE_UP: &str = "page_up";
    pub const PUSH: &str = "push";
    pub const QUIT: &str = "quit";
    pub const REBASE_DESC: &str = "rebase_desc";
    pub const REBASE_SINGLE: &str = "rebase_single";
    pub const RESOLVE: &str = "resolve";
    pub const RUN: &str = "run";
    pub const SCROLL_DOWN: &str = "scroll_down";
    pub const SCROLL_UP: &str = "scroll_up";
    pub const SELECT: &str = "select";
    pub const STACK_SYNC: &str = "stack_sync";
    pub const TOGGLE: &str = "toggle";
    pub const TOP: &str = "top";
    pub const TRUNK_SINGLE: &str = "trunk_single";
    pub const UP: &str = "up";
    pub const ZOOM: &str = "zoom";
    pub const ALL: &str = "all";
    pub const NONE: &str = "none";
    pub const CONFIRM: &str = "confirm";
    pub const CANCEL: &str = "cancel";
    pub const BRANCHES: &str = "branches";
    pub const CLOSE: &str = "close";
    pub const NAV: &str = "nav";
    pub const NO: &str = "no";
    pub const YES: &str = "yes";
}

pub fn command_specs() -> Vec<CommandSpec> {
    bindings::builtin_commands()
}

pub(crate) use hints::{DynamicHintValue, HintScenario, HintSpec, hint_specs};
