//! Controller module for jj_tui
//!
//! The controller maps (ModeState, KeyEvent) → Action.
use super::action::Action;
use super::keybindings;
use super::state::ModeState;
use ratatui::crossterm::event::KeyEvent;

pub struct ControllerContext<'a> {
    pub mode: &'a ModeState,
    pub pending_key: Option<char>,
    pub viewport_height: usize,
    pub has_focus: bool,
    pub has_selection: bool,
    pub neighborhood_active: bool,
    pub has_neighborhood_history: bool,
    pub can_enter_neighborhood_path: bool,
}

/// Map a key event to an action based on current mode
pub fn handle_key(ctx: &ControllerContext, key: KeyEvent) -> Action {
    keybindings::dispatch_key(ctx, key)
}
