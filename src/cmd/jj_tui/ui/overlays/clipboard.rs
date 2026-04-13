use super::common::{centered_popup_area, empty_line, footer_line, render_popup_shell, short_rev};
use crate::cmd::jj_tui::state::ClipboardBranchSelectState;
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn render_clipboard_branch_select(
    frame: &mut Frame,
    state: &ClipboardBranchSelectState,
) {
    let area = frame.area();
    let popup_width = 56u16.min(area.width.saturating_sub(4));
    let popup_height = (6 + state.options.len().min(10)) as u16;
    let popup_area = centered_popup_area(area, popup_width, popup_height);
    let inner = render_popup_shell(
        frame,
        popup_area,
        " Copy Branch ",
        Color::Cyan,
        crate::cmd::jj_tui::theme::POPUP_BG,
    );

    let mut lines = vec![target_line(&state.target_rev), empty_line()];
    for option in state.options.iter().take(10) {
        lines.push(option_line(option.key, &option.branch));
    }
    if state.options.len() > 10 {
        lines.push(Line::from(Span::styled(
            format!("  ... and {} more", state.options.len() - 10),
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(empty_line());
    lines.push(footer_line(
        "press letter to copy | Esc: cancel".to_string(),
    ));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn target_line(target_rev: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("At: ", Style::default().fg(Color::Yellow)),
        Span::styled(short_rev(target_rev), Style::default().fg(Color::DarkGray)),
    ])
}

fn option_line(key: char, branch: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{key}  "),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(branch.to_string(), Style::default().fg(Color::White)),
    ])
}
