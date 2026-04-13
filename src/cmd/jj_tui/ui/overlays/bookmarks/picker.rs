use super::super::common::{
    centered_popup_area, empty_line, filter_input_line, render_popup_shell,
};
use super::{append_overflow, dimmed_line, picker_footer, select_popup_style};
use crate::cmd::jj_tui::state::{BookmarkPickerState, BookmarkSelectAction};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

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
        lines.push(move_target_line(&state.target_rev));
        lines.push(empty_line());
    }

    lines.push(filter_input_line(&state.filter, placeholder(state.action)));
    lines.push(empty_line());
    append_picker_items(&mut lines, state, &filtered);
    lines.push(empty_line());
    lines.push(picker_footer(state.action));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn move_target_line(target_rev: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("Set on: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            super::short_rev(target_rev),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

fn placeholder(action: BookmarkSelectAction) -> &'static str {
    match action {
        BookmarkSelectAction::Move => "type to filter or create...",
        BookmarkSelectAction::Delete | BookmarkSelectAction::CreatePR => "type to filter...",
    }
}

fn append_picker_items(
    lines: &mut Vec<Line<'static>>,
    state: &BookmarkPickerState,
    filtered: &[&String],
) {
    if filtered.is_empty() {
        lines.push(no_picker_match_line(state));
        return;
    }

    for (index, bookmark) in filtered.iter().take(10).enumerate() {
        lines.push(super::selectable_item(
            index == state.selected_index,
            (*bookmark).clone(),
        ));
    }
    append_overflow(lines, filtered.len(), 10);
}

fn no_picker_match_line(state: &BookmarkPickerState) -> Line<'static> {
    if matches!(state.action, BookmarkSelectAction::Move) && !state.filter.is_empty() {
        return dimmed_line(format!("  create '{}'", state.filter));
    }

    dimmed_line("  (no matching bookmarks)")
}
