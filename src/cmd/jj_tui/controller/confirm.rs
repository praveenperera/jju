//! Confirmation dialog key handling

use super::super::action::Action;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in confirmation dialog
pub fn handle(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => Action::ConfirmYes,
        KeyCode::Char('n') | KeyCode::Esc => Action::ConfirmNo,
        _ => Action::Noop,
    }
}
