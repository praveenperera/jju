//! Selection mode key handling

use super::super::action::Action;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in visual selection mode
/// Note: The engine handles move + extend as a combined operation
pub fn handle(key: KeyEvent) -> Action {
    match key.code {
        // in selection mode, j/k move cursor AND extend selection
        KeyCode::Char('j') | KeyCode::Down => Action::MoveCursorDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveCursorUp,
        KeyCode::Esc => Action::ExitSelecting,
        KeyCode::Char('a') => Action::EnterConfirmAbandon,
        _ => Action::Noop,
    }
}
