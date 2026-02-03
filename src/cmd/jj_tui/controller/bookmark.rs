//! Bookmark mode key handling
//!
//! Handles all four bookmark-related modes:
//! - MovingBookmark: moving a bookmark to a different revision
//! - BookmarkInput: creating a new bookmark
//! - BookmarkSelect: selecting from bookmarks on current revision
//! - BookmarkPicker: selecting from all bookmarks with filter

use super::super::action::Action;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handle keys in moving bookmark mode
pub fn handle_moving(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::MoveBookmarkDestDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveBookmarkDestUp,
        KeyCode::Enter => Action::ExecuteBookmarkMove,
        KeyCode::Esc => Action::ExitBookmarkMode,
        _ => Action::Noop,
    }
}

/// Handle keys in bookmark input mode (creating new bookmark)
pub fn handle_input(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Enter => Action::ConfirmBookmarkInput,
        KeyCode::Esc => Action::ExitBookmarkMode,
        KeyCode::Char(c) => Action::BookmarkInputChar(c),
        KeyCode::Backspace => Action::BookmarkInputBackspace,
        KeyCode::Delete => Action::BookmarkInputDelete,
        KeyCode::Left => Action::BookmarkInputCursorLeft,
        KeyCode::Right => Action::BookmarkInputCursorRight,
        _ => Action::Noop,
    }
}

/// Handle keys in bookmark select mode (choosing from bookmarks on revision)
pub fn handle_select(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::SelectBookmarkDown,
        KeyCode::Char('k') | KeyCode::Up => Action::SelectBookmarkUp,
        KeyCode::Enter => Action::ConfirmBookmarkSelect,
        KeyCode::Esc => Action::ExitBookmarkMode,
        _ => Action::Noop,
    }
}

/// Handle keys in bookmark picker mode (choosing from all bookmarks with filter)
pub fn handle_picker(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ExitBookmarkMode,
        KeyCode::Enter => Action::ConfirmBookmarkPicker,
        KeyCode::Down => Action::BookmarkPickerDown,
        KeyCode::Up => Action::BookmarkPickerUp,
        KeyCode::Char(c) => Action::BookmarkFilterChar(c),
        KeyCode::Backspace => Action::BookmarkFilterBackspace,
        _ => Action::Noop,
    }
}
