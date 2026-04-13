mod picker;
mod push_select;
mod select;

use super::common::{
    footer_line, key_hint, overflow_line, popup_bg, popup_bg_delete, selectable_item, short_rev,
};
use crate::cmd::jj_tui::{
    keybindings,
    state::{BookmarkPickerState, BookmarkSelectAction, BookmarkSelectState, PushSelectState},
};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub(super) fn render_bookmark_select(frame: &mut Frame, state: &BookmarkSelectState) {
    select::render_bookmark_select(frame, state);
}

pub(super) fn render_bookmark_picker(frame: &mut Frame, state: &BookmarkPickerState) {
    picker::render_bookmark_picker(frame, state);
}

pub(super) fn render_push_select(frame: &mut Frame, state: &PushSelectState) {
    push_select::render_push_select(frame, state);
}

fn select_popup_style(action: BookmarkSelectAction) -> (&'static str, Color, Color) {
    match action {
        BookmarkSelectAction::Move => (" Select Bookmark to Move ", Color::Cyan, popup_bg()),
        BookmarkSelectAction::Delete => {
            (" Select Bookmark to Delete ", Color::Red, popup_bg_delete())
        }
        BookmarkSelectAction::CreatePR => (" PR from Bookmark ", Color::Green, popup_bg()),
    }
}

fn append_overflow(lines: &mut Vec<Line<'static>>, total: usize, visible_limit: usize) {
    if total > visible_limit {
        lines.push(overflow_line(total - visible_limit));
    }
}

fn dimmed_line(text: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(
        text.into(),
        Style::default().fg(Color::DarkGray),
    ))
}

fn picker_footer(action: BookmarkSelectAction) -> Line<'static> {
    let up_key = key_hint(keybindings::ModeId::BookmarkPicker, "up", false);
    let down_key = key_hint(keybindings::ModeId::BookmarkPicker, "down", false);
    let confirm_key = key_hint(keybindings::ModeId::BookmarkPicker, "confirm", false);
    let cancel_key = key_hint(keybindings::ModeId::BookmarkPicker, "cancel", false);

    let text = match action {
        BookmarkSelectAction::Move => format!(
            "type: filter/create | {up_key}/{down_key}: navigate | {confirm_key}: set | {cancel_key}: cancel"
        ),
        BookmarkSelectAction::Delete => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: delete | {cancel_key}: cancel"
        ),
        BookmarkSelectAction::CreatePR => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: PR | {cancel_key}: cancel"
        ),
    };
    footer_line(text)
}

fn push_select_footer() -> Line<'static> {
    let up_key = key_hint(keybindings::ModeId::PushSelect, "up", false);
    let down_key = key_hint(keybindings::ModeId::PushSelect, "down", false);
    let toggle_key = key_hint(keybindings::ModeId::PushSelect, "toggle", false);
    let all_key = key_hint(keybindings::ModeId::PushSelect, "all", false);
    let none_key = key_hint(keybindings::ModeId::PushSelect, "none", false);
    let push_key = key_hint(keybindings::ModeId::PushSelect, "push", false);
    let cancel_key = key_hint(keybindings::ModeId::PushSelect, "cancel", false);

    footer_line(format!(
        "{up_key}/{down_key}: nav | {toggle_key}: toggle | {all_key}/{none_key}: all/none | {push_key}: push | {cancel_key}: cancel"
    ))
}

fn push_select_item(is_cursor: bool, is_selected: bool, bookmark: &str) -> Line<'static> {
    let marker = if is_cursor { "> " } else { "  " };
    let checkbox = if is_selected { "[x] " } else { "[ ] " };
    let style = if is_cursor {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::White)
    };

    Line::from(Span::styled(format!("{marker}{checkbox}{bookmark}"), style))
}
