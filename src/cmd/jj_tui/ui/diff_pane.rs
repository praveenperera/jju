use crate::cmd::jj_tui::app::App;
use crate::cmd::jj_tui::keybindings;
use crate::cmd::jj_tui::state::{DiffLineKind, DiffState};
use crate::cmd::jj_tui::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub(crate) fn render_diff(frame: &mut Frame, state: &DiffState, area: Rect) {
    let block = Block::default()
        .title(format!(" Diff: {} ", state.rev))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let viewport_height = inner.height as usize;
    let lines: Vec<Line> = state
        .lines
        .iter()
        .skip(state.scroll_offset)
        .take(viewport_height)
        .map(|diff_line| {
            let bg = match diff_line.kind {
                DiffLineKind::Added => Some(theme::DIFF_ADDED_BG),
                DiffLineKind::Removed => Some(theme::DIFF_REMOVED_BG),
                _ => None,
            };

            let spans: Vec<Span> = diff_line
                .spans
                .iter()
                .map(|span| {
                    let mut style = Style::default().fg(span.fg);
                    if let Some(bg_color) = bg {
                        style = style.bg(bg_color);
                    }
                    Span::styled(span.text.clone(), style)
                })
                .collect();

            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

pub(crate) fn render_diff_pane(frame: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Diff ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let diff_key = keybindings::display_keys_joined(
        keybindings::ModeId::Normal,
        None,
        "diff",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let hint = Paragraph::new(format!("Press {diff_key} to view diff"))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, inner);
}
