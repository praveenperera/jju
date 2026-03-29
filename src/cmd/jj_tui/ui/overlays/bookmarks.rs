use super::common::{
    centered_popup_area, empty_line, filter_input_line, footer_line, key_hint, overflow_line,
    popup_bg, popup_bg_delete, render_popup_shell, selectable_item, short_rev,
};
use crate::cmd::jj_tui::{
    keybindings,
    state::{BookmarkPickerState, BookmarkSelectAction, BookmarkSelectState, PushSelectState},
};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn render_bookmark_select(frame: &mut Frame, state: &BookmarkSelectState) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = (6 + state.bookmarks.len().min(10)) as u16;
    let popup_area = centered_popup_area(area, popup_width, popup_height);

    let (title, border_color, bg_color) = select_popup_style(state.action);
    let inner = render_popup_shell(frame, popup_area, title, border_color, bg_color);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("At: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                short_rev(&state.target_rev),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        empty_line(),
    ];

    for (index, bookmark) in state.bookmarks.iter().enumerate() {
        lines.push(selectable_item(
            index == state.selected_index,
            bookmark.clone(),
        ));
    }

    let down_key = key_hint(keybindings::ModeId::BookmarkSelect, "down", false);
    let up_key = key_hint(keybindings::ModeId::BookmarkSelect, "up", false);
    let select_key = key_hint(keybindings::ModeId::BookmarkSelect, "select", false);
    let cancel_key = key_hint(keybindings::ModeId::BookmarkSelect, "cancel", false);
    lines.push(empty_line());
    lines.push(footer_line(format!(
        "{down_key}/{up_key}: navigate | {select_key}: select | {cancel_key}: cancel"
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

pub(super) fn render_bookmark_picker(frame: &mut Frame, state: &BookmarkPickerState) {
    let area = frame.area();
    let filtered = state.filtered_bookmarks();
    let list_height = filtered.len().min(10);
    let popup_height = (8 + list_height) as u16;
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_area = centered_popup_area(area, popup_width, popup_height);

    let (title, border_color, bg_color) = select_popup_style(state.action);
    let inner = render_popup_shell(frame, popup_area, title, border_color, bg_color);

    let mut lines: Vec<Line> = Vec::new();
    if matches!(state.action, BookmarkSelectAction::Move) {
        lines.push(Line::from(vec![
            Span::styled("Set on: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                short_rev(&state.target_rev),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(empty_line());
    }

    let placeholder = match state.action {
        BookmarkSelectAction::Move => "type to filter or create...",
        BookmarkSelectAction::Delete | BookmarkSelectAction::CreatePR => "type to filter...",
    };
    lines.push(filter_input_line(&state.filter, placeholder));
    lines.push(empty_line());

    append_picker_items(&mut lines, state, &filtered);
    lines.push(empty_line());
    lines.push(footer_line(picker_footer(state.action)));

    frame.render_widget(Paragraph::new(lines), inner);
}

pub(super) fn render_push_select(frame: &mut Frame, state: &PushSelectState) {
    let area = frame.area();
    let filtered = state.filtered_bookmarks();
    let list_height = filtered.len().clamp(1, 10);
    let popup_height = (8 + list_height) as u16;
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_area = centered_popup_area(area, popup_width, popup_height);
    let inner = render_popup_shell(
        frame,
        popup_area,
        " Push Bookmarks ",
        Color::Cyan,
        popup_bg(),
    );

    let selected_count = state.selected_filtered_count();
    let total_filtered = filtered.len();
    let mut lines = vec![
        Line::from(Span::styled(
            format!("Select bookmarks to push: {selected_count}/{total_filtered} selected"),
            Style::default().fg(Color::Yellow),
        )),
        empty_line(),
        filter_input_line(&state.filter, "type to filter..."),
        empty_line(),
    ];

    append_push_items(&mut lines, state, &filtered);
    lines.push(empty_line());
    lines.push(footer_line(push_select_footer()));

    frame.render_widget(Paragraph::new(lines), inner);
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

fn append_picker_items(
    lines: &mut Vec<Line<'static>>,
    state: &BookmarkPickerState,
    filtered: &[&String],
) {
    if filtered.is_empty() {
        lines.push(dimmed_line(no_picker_match_text(state)));
        return;
    }

    append_selectable_items(
        lines,
        filtered.iter().take(10).enumerate(),
        |index, bookmark| selectable_item(index == state.selected_index, (*bookmark).clone()),
    );
    append_overflow(lines, filtered.len(), 10);
}

fn append_push_items(
    lines: &mut Vec<Line<'static>>,
    state: &PushSelectState,
    filtered: &[(usize, &str)],
) {
    if filtered.is_empty() {
        lines.push(dimmed_line("  (no matching bookmarks)".to_string()));
        return;
    }

    append_selectable_items(
        lines,
        filtered.iter().take(10).enumerate(),
        |display_index, item| push_select_item(state, display_index, item),
    );
    append_overflow(lines, filtered.len(), 10);
}

fn append_selectable_items<T, I, F>(lines: &mut Vec<Line<'static>>, items: I, row: F)
where
    I: IntoIterator<Item = (usize, T)>,
    F: Fn(usize, T) -> Line<'static>,
{
    for (index, item) in items {
        lines.push(row(index, item));
    }
}

fn append_overflow(lines: &mut Vec<Line<'static>>, total: usize, visible_limit: usize) {
    if total > visible_limit {
        lines.push(overflow_line(total - visible_limit));
    }
}

fn no_picker_match_text(state: &BookmarkPickerState) -> String {
    if matches!(state.action, BookmarkSelectAction::Move) && !state.filter.is_empty() {
        format!("  create '{}'", state.filter)
    } else {
        "  (no matching bookmarks)".to_string()
    }
}

fn dimmed_line(text: String) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::DarkGray)))
}

fn picker_footer(action: BookmarkSelectAction) -> String {
    let up_key = key_hint(keybindings::ModeId::BookmarkPicker, "up", false);
    let down_key = key_hint(keybindings::ModeId::BookmarkPicker, "down", false);
    let confirm_key = key_hint(keybindings::ModeId::BookmarkPicker, "confirm", false);
    let cancel_key = key_hint(keybindings::ModeId::BookmarkPicker, "cancel", false);

    match action {
        BookmarkSelectAction::Move => format!(
            "type: filter/create | {up_key}/{down_key}: navigate | {confirm_key}: set | {cancel_key}: cancel"
        ),
        BookmarkSelectAction::Delete => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: delete | {cancel_key}: cancel"
        ),
        BookmarkSelectAction::CreatePR => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: PR | {cancel_key}: cancel"
        ),
    }
}

fn push_select_footer() -> String {
    let up_key = key_hint(keybindings::ModeId::PushSelect, "up", false);
    let down_key = key_hint(keybindings::ModeId::PushSelect, "down", false);
    let toggle_key = key_hint(keybindings::ModeId::PushSelect, "toggle", false);
    let all_key = key_hint(keybindings::ModeId::PushSelect, "all", false);
    let none_key = key_hint(keybindings::ModeId::PushSelect, "none", false);
    let push_key = key_hint(keybindings::ModeId::PushSelect, "push", false);
    let cancel_key = key_hint(keybindings::ModeId::PushSelect, "cancel", false);

    format!(
        "{up_key}/{down_key}: nav | {toggle_key}: toggle | {all_key}/{none_key}: all/none | {push_key}: push | {cancel_key}: cancel"
    )
}

fn push_select_item(
    state: &PushSelectState,
    display_index: usize,
    (original_index, bookmark): &(usize, &str),
) -> Line<'static> {
    let is_cursor = display_index == state.cursor_index;
    let is_selected = state.selected.contains(original_index);
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
