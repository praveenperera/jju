use crate::cmd::jj_tui::{keybindings, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear},
};

pub(super) fn centered_popup_area(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: (area.width.saturating_sub(width)) / 2,
        y: (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

pub(super) fn render_popup_shell(
    frame: &mut Frame,
    popup_area: Rect,
    title: &str,
    border_color: Color,
    bg_color: Color,
) -> Rect {
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(popup_area);

    frame.render_widget(block.style(Style::default().bg(bg_color)), popup_area);

    inner
}

pub(super) fn key_hint(mode: keybindings::ModeId, action: &str, include_aliases: bool) -> String {
    keybindings::display_keys_joined(
        mode,
        None,
        action,
        include_aliases,
        keybindings::KeyFormat::Space,
        "/",
    )
}

pub(super) fn short_rev(rev: &str) -> String {
    rev.chars().take(8).collect()
}

pub(super) fn selectable_item(is_selected: bool, text: String) -> Line<'static> {
    let marker = if is_selected { "> " } else { "  " };
    let style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    Line::from(Span::styled(format!("{marker}{text}"), style))
}

pub(super) fn filter_input_line(filter: &str, placeholder: &str) -> Line<'static> {
    let filter_display = if filter.is_empty() {
        placeholder.to_string()
    } else {
        filter.to_string()
    };
    let filter_style = if filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    Line::from(vec![
        Span::styled("Filter: ", Style::default().fg(Color::Green)),
        Span::styled(filter_display, filter_style),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ])
}

pub(super) fn overflow_line(hidden_count: usize) -> Line<'static> {
    Line::from(Span::styled(
        format!("  ... and {hidden_count} more"),
        Style::default().fg(Color::DarkGray),
    ))
}

pub(super) fn empty_line() -> Line<'static> {
    Line::from("")
}

pub(super) fn footer_line(text: String) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::DarkGray)))
}

pub(super) fn popup_bg_delete() -> Color {
    theme::POPUP_BG_DELETE
}

pub(super) fn popup_bg() -> Color {
    theme::POPUP_BG
}
