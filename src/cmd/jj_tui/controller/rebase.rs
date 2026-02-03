//! Rebase mode key handling

use super::super::action::Action;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in rebasing mode
pub fn handle(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::MoveRebaseDestDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveRebaseDestUp,
        KeyCode::Char('b') => Action::ToggleRebaseBranches,
        KeyCode::Enter => Action::ExecuteRebase,
        KeyCode::Esc => Action::ExitRebaseMode,
        _ => Action::Noop,
    }
}
