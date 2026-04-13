use crate::cmd::jj_tui::keybindings;
use crate::cmd::jj_tui::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

const KEY_SPACING: usize = 3;
const TITLE_WRAPPER: usize = 4;
const HORIZONTAL_PADDING: usize = 4;
const FIXED_HEIGHT_LINES: usize = 4;
const EDGE_MARGIN: u16 = 2;

pub(super) fn render_prefix_key_popup(
    frame: &mut Frame,
    mode: keybindings::ModeId,
    pending_key: char,
) {
    let Some(menu) = keybindings::prefix_menu(mode, pending_key) else {
        return;
    };

    let popup_area = popup_area(frame.area(), menu.title, &menu.items);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(theme::PREFIX_POPUP_BG));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    frame.render_widget(
        Paragraph::new(prefix_lines(
            pending_key,
            menu.title,
            menu.items,
            inner.width,
        )),
        inner,
    );
}

fn popup_area(area: Rect, title: &str, items: &[(String, &'static str)]) -> Rect {
    let max_label_width = items
        .iter()
        .map(|(_, label)| label.width())
        .max()
        .unwrap_or(0);
    let content_width = max_label_width + KEY_SPACING;
    let title_width = title.width() + TITLE_WRAPPER;
    let popup_width = (content_width.max(title_width) + HORIZONTAL_PADDING) as u16;
    let popup_height = (items.len() + FIXED_HEIGHT_LINES) as u16;

    Rect {
        x: area.width.saturating_sub(popup_width + EDGE_MARGIN),
        y: area.height.saturating_sub(popup_height + EDGE_MARGIN),
        width: popup_width,
        height: popup_height,
    }
}

fn prefix_lines(
    pending_key: char,
    title: &str,
    items: Vec<(String, &'static str)>,
    inner_width: u16,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        format!("{pending_key} ({title})"),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(Span::styled(
        "─".repeat(inner_width as usize),
        Style::default().fg(Color::DarkGray),
    )));

    for (key, label) in items {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{key}  "),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(label, Style::default().fg(Color::White)),
        ]));
    }

    lines
}
