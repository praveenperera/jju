//! Squash mode key handling

use super::super::action::Action;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in squashing mode
pub fn handle(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::MoveSquashDestDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveSquashDestUp,
        KeyCode::Enter => Action::ExecuteSquash,
        KeyCode::Esc => Action::ExitSquashMode,
        _ => Action::Noop,
    }
}
