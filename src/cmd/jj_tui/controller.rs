//! Controller module for jj_tui
//!
//! The controller maps (ModeState, KeyEvent) → Action.
//! Each mode has its own handler module.

mod bookmark;
mod confirm;
mod conflicts;
mod diff;
mod normal;
mod rebase;
mod selection;
mod squash;

use super::action::Action;
use super::state::ModeState;
use ratatui::crossterm::event::KeyEvent;

/// Context passed to controllers for read-only state access
pub struct ControllerContext<'a> {
    pub mode: &'a ModeState,
    pub pending_key: Option<char>,
    pub viewport_height: usize,
    pub has_focus: bool,
    pub has_selection: bool,
}

/// Map a key event to an action based on current mode
pub fn handle_key(ctx: &ControllerContext, key: KeyEvent) -> Action {
    match ctx.mode {
        ModeState::Normal => normal::handle(ctx, key),
        ModeState::Help => normal::handle_help(key),
        ModeState::ViewingDiff(_) => diff::handle(ctx, key),
        ModeState::Confirming(_) => confirm::handle(key),
        ModeState::Selecting => selection::handle(key),
        ModeState::Rebasing(_) => rebase::handle(key),
        ModeState::MovingBookmark(_) => bookmark::handle_moving(key),
        ModeState::BookmarkInput(_) => bookmark::handle_input(key),
        ModeState::BookmarkSelect(_) => bookmark::handle_select(key),
        ModeState::BookmarkPicker(_) => bookmark::handle_picker(key),
        ModeState::Squashing(_) => squash::handle(key),
        ModeState::Conflicts(_) => conflicts::handle(ctx, key),
    }
}
