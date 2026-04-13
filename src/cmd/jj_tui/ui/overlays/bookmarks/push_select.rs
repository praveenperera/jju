use super::super::common::{
    centered_popup_area, empty_line, filter_input_line, popup_bg, render_popup_shell,
};
use super::{append_overflow, dimmed_line, push_select_footer, push_select_item};
use crate::cmd::jj_tui::state::PushSelectState;
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

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
        summary_line(selected_count, total_filtered),
        empty_line(),
        filter_input_line(&state.filter, "type to filter..."),
        empty_line(),
    ];

    append_push_items(&mut lines, state, &filtered);
    lines.push(empty_line());
    lines.push(push_select_footer());

    frame.render_widget(Paragraph::new(lines), inner);
}

fn summary_line(selected_count: usize, total_filtered: usize) -> Line<'static> {
    Line::from(Span::styled(
        format!("Select bookmarks to push: {selected_count}/{total_filtered} selected"),
        Style::default().fg(Color::Yellow),
    ))
}

fn append_push_items(
    lines: &mut Vec<Line<'static>>,
    state: &PushSelectState,
    filtered: &[(usize, &str)],
) {
    if filtered.is_empty() {
        lines.push(dimmed_line("  (no matching bookmarks)"));
        return;
    }

    for (display_index, (original_index, bookmark)) in filtered.iter().take(10).enumerate() {
        lines.push(push_select_item(
            display_index == state.cursor_index,
            state.selected.contains(original_index),
            bookmark,
        ));
    }
    append_overflow(lines, filtered.len(), 10);
}
