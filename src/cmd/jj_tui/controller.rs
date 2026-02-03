//! Controller module for jj_tui
//!
//! The controller maps (ModeState, KeyEvent) → Action.
//! Each mode has its own handler module.
use super::action::Action;
use super::keybindings;
use ratatui::crossterm::event::KeyEvent;

pub use keybindings::ControllerContext;

/// Map a key event to an action based on current mode
pub fn handle_key(ctx: &ControllerContext, key: KeyEvent) -> Action {
    keybindings::handle_key(ctx, key)
}
