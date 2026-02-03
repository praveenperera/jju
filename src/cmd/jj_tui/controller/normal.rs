//! Normal mode key handling

use super::super::action::Action;
use super::super::state::RebaseType;
use super::ControllerContext;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keys in Normal mode
pub fn handle(ctx: &ControllerContext, key: KeyEvent) -> Action {
    let code = key.code;
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // handle pending key sequences
    if let Some(pending) = ctx.pending_key {
        return handle_pending(pending, code, ctx.viewport_height);
    }

    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('Q') => Action::EnterSquashMode,
        KeyCode::Esc => {
            if ctx.has_focus {
                Action::Unfocus
            } else if ctx.has_selection {
                Action::ClearSelection
            } else {
                Action::Noop
            }
        }
        KeyCode::Char('?') => Action::EnterHelp,

        KeyCode::Char('j') | KeyCode::Down => Action::MoveCursorDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveCursorUp,
        KeyCode::Char('@') => Action::JumpToWorkingCopy,

        // multi-key sequence prefixes
        KeyCode::Char('g') => Action::SetPendingKey('g'),
        KeyCode::Char('z') => Action::SetPendingKey('z'),
        KeyCode::Char('b') => Action::SetPendingKey('b'),

        KeyCode::Char('f') => Action::ToggleFullMode,

        // zoom in/out on node
        KeyCode::Enter => Action::ToggleFocus,

        // details toggle
        KeyCode::Tab | KeyCode::Char(' ') => Action::ToggleExpanded,

        // page scrolling
        KeyCode::Char('u') if ctrl => Action::PageUp(ctx.viewport_height / 2),
        KeyCode::Char('d') if ctrl => Action::PageDown(ctx.viewport_height / 2),

        // split view toggle
        KeyCode::Char('\\') => Action::ToggleSplitView,

        // diff viewing
        KeyCode::Char('d') => Action::EnterDiffView,
        KeyCode::Char('D') => Action::EditDescription,
        KeyCode::Char('e') => Action::EditWorkingCopy,
        KeyCode::Char('n') => Action::CreateNewCommit,
        KeyCode::Char('c') if ctrl => Action::Quit,
        KeyCode::Char('c') => Action::CommitWorkingCopy,

        // selection
        KeyCode::Char('x') => Action::ToggleSelection,
        KeyCode::Char('v') => Action::EnterSelecting,
        KeyCode::Char('a') => Action::EnterConfirmAbandon,

        // rebase operations
        KeyCode::Char('r') => Action::EnterRebaseMode(RebaseType::Single),
        KeyCode::Char('s') => Action::EnterRebaseMode(RebaseType::WithDescendants),
        KeyCode::Char('t') => Action::EnterConfirmRebaseOntoTrunk(RebaseType::Single),
        KeyCode::Char('T') => Action::EnterConfirmRebaseOntoTrunk(RebaseType::WithDescendants),

        // undo
        KeyCode::Char('u') => Action::Undo,

        // git push
        KeyCode::Char('p') => Action::GitPush,
        KeyCode::Char('P') => Action::GitPushAll,

        // conflicts panel
        KeyCode::Char('C') => Action::EnterConflicts,

        _ => Action::Noop,
    }
}

/// Handle pending key sequences
fn handle_pending(pending: char, code: KeyCode, viewport_height: usize) -> Action {
    match (pending, code) {
        // 'g' prefix - git operations
        ('g', KeyCode::Char('i')) => Action::GitImport,
        ('g', KeyCode::Char('e')) => Action::GitExport,
        // 'z' prefix - navigation
        ('z', KeyCode::Char('t')) => Action::MoveCursorTop,
        ('z', KeyCode::Char('b')) => Action::MoveCursorBottom,
        ('z', KeyCode::Char('z')) => Action::CenterCursor(viewport_height),
        // 'b' prefix - bookmark operations
        ('b', KeyCode::Char('m')) => Action::EnterMoveBookmarkMode,
        ('b', KeyCode::Char('s')) => Action::EnterCreateBookmark,
        ('b', KeyCode::Char('d')) => Action::EnterBookmarkPicker(
            super::super::state::BookmarkSelectAction::Delete,
        ),
        // any other key after prefix - clear pending
        _ => Action::ClearPendingKey,
    }
}

/// Handle keys in Help mode
pub fn handle_help(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q' | '?') | KeyCode::Esc => Action::ExitHelp,
        _ => Action::Noop,
    }
}
