//! Conflicts panel key handling

use super::super::action::Action;
use super::ControllerContext;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in conflicts panel mode
pub fn handle(_ctx: &ControllerContext, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::ConflictsDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ConflictsUp,
        KeyCode::Enter => Action::ConflictsJump,
        KeyCode::Char('R') => Action::StartResolveFromConflicts,
        KeyCode::Esc | KeyCode::Char('q') => Action::ExitConflicts,
        _ => Action::Noop,
    }
}

