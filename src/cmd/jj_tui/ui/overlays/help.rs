use super::common::centered_popup_area;
use crate::cmd::jj_tui::keybindings;
use crate::cmd::jj_tui::state::HelpState;
use crate::cmd::jj_tui::theme;
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

pub(super) fn render_help(frame: &mut Frame, help_state: &HelpState) {
    let view = keybindings::build_help_view();
    let key_col_width = view
        .iter()
        .flat_map(|section| section.items.iter().map(|item| item.keys.width()))
        .max()
        .unwrap_or(9)
        .max(9);
    let mut help_text = Vec::new();

    for (section_idx, section) in view.iter().enumerate() {
        if section_idx > 0 {
            help_text.push(Line::from(""));
        }
        help_text.push(Line::from(Span::styled(
            section.title,
            Style::default().add_modifier(Modifier::BOLD),
        )));

        for item in &section.items {
            let pad_len = key_col_width.saturating_sub(item.keys.width());
            let pad = " ".repeat(pad_len);
            help_text.push(Line::from(format!(
                "  {}{}  {}",
                item.keys, pad, item.description
            )));
        }
    }

    let area = frame.area();
    let content_width = help_text
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.width())
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0) as u16;
    let content_height = help_text.len() as u16;
    let popup_width = (content_width + 4).min(area.width.saturating_sub(4));
    let popup_height = (content_height + 2).min(area.height.saturating_sub(4));
    let popup_area = centered_popup_area(area, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(
        Paragraph::new(help_text)
            .scroll((help_state.scroll_offset as u16, 0))
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().bg(theme::POPUP_BG)),
        popup_area,
    );
}
