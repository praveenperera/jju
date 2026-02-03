//! Diff view mode key handling

use super::super::action::Action;
use super::ControllerContext;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in diff viewing mode
pub fn handle(ctx: &ControllerContext, key: KeyEvent) -> Action {
    // handle pending key sequences in diff view
    if let Some(pending) = ctx.pending_key {
        return handle_pending(pending, key.code);
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDiffDown(1),
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollDiffUp(1),
        KeyCode::Char('d') => Action::ScrollDiffDown(20),
        KeyCode::Char('u') => Action::ScrollDiffUp(20),
        KeyCode::Char('z') => Action::SetPendingKey('z'),
        KeyCode::Esc | KeyCode::Char('q') => Action::ExitDiffView,
        _ => Action::Noop,
    }
}

/// Handle pending key sequences in diff view
fn handle_pending(pending: char, code: KeyCode) -> Action {
    match (pending, code) {
        ('z', KeyCode::Char('t')) => Action::ScrollDiffTop,
        ('z', KeyCode::Char('b')) => Action::ScrollDiffBottom,
        _ => Action::ClearPendingKey,
    }
}
