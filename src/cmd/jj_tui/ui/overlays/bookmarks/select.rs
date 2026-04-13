use super::super::common::{
    centered_popup_area, empty_line, footer_line, key_hint, render_popup_shell,
};
use super::{select_popup_style, selectable_item, short_rev};
use crate::cmd::jj_tui::{keybindings, state::BookmarkSelectState};
use ratatui::{
    Frame,
    style::{Color, Style},
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

    let mut lines = vec![target_line(&state.target_rev), empty_line()];

    for (index, bookmark) in state.bookmarks.iter().enumerate() {
        lines.push(selectable_item(
            index == state.selected_index,
            bookmark.clone(),
        ));
    }

    lines.push(empty_line());
    lines.push(selection_footer());

    frame.render_widget(Paragraph::new(lines), inner);
}

fn target_line(target_rev: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("At: ", Style::default().fg(Color::Yellow)),
        Span::styled(short_rev(target_rev), Style::default().fg(Color::DarkGray)),
    ])
}

fn selection_footer() -> Line<'static> {
    let down_key = key_hint(keybindings::ModeId::BookmarkSelect, "down", false);
    let up_key = key_hint(keybindings::ModeId::BookmarkSelect, "up", false);
    let select_key = key_hint(keybindings::ModeId::BookmarkSelect, "select", false);
    let cancel_key = key_hint(keybindings::ModeId::BookmarkSelect, "cancel", false);

    footer_line(format!(
        "{down_key}/{up_key}: navigate | {select_key}: select | {cancel_key}: cancel"
    ))
}
