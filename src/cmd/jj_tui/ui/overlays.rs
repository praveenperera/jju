mod bookmarks;
mod common;
mod confirm;
mod conflicts;
mod help;
mod prefix;
mod toast;

use super::super::app::App;
use super::super::keybindings;
use super::super::state::ModeState;
use bookmarks::{render_bookmark_picker, render_bookmark_select, render_push_select};
use confirm::render_confirmation;
use conflicts::render_conflicts_panel;
use help::render_help;
use prefix::render_prefix_key_popup;
use ratatui::Frame;
use toast::render_toast;

pub(super) fn render_overlays(frame: &mut Frame, app: &App) {
    if let ModeState::Help(ref help_state) = app.mode {
        render_help(frame, help_state);
    }

    if let ModeState::Confirming(ref state) = app.mode {
        render_confirmation(frame, state);
    }

    if let ModeState::BookmarkSelect(ref state) = app.mode {
        render_bookmark_select(frame, state);
    }

    if let ModeState::BookmarkPicker(ref state) = app.mode {
        render_bookmark_picker(frame, state);
    }

    if let ModeState::PushSelect(ref state) = app.mode {
        render_push_select(frame, state);
    }

    if let ModeState::Conflicts(ref state) = app.mode {
        render_conflicts_panel(frame, state);
    }

    if let Some(pending) = app.pending_key {
        render_prefix_key_popup(frame, keybindings::mode_id_from_state(&app.mode), pending);
    }

    if let Some(ref msg) = app.status_message
        && std::time::Instant::now() < msg.expires
    {
        render_toast(frame, msg);
    }
}
