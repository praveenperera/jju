use super::common::{centered_popup_area, key_hint, render_popup_shell, selectable_item};
use crate::cmd::jj_tui::keybindings;
use crate::cmd::jj_tui::state::ConflictsState;
use crate::cmd::jj_tui::theme;
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn render_conflicts_panel(frame: &mut Frame, state: &ConflictsState) {
    let area = frame.area();
    let list_height = state.files.len().clamp(1, 15);
    let popup_height = (5 + list_height) as u16;
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_area = centered_popup_area(area, popup_width, popup_height);
    let inner = render_popup_shell(
        frame,
        popup_area,
        " Conflicted Files ",
        Color::Red,
        theme::POPUP_BG,
    );

    frame.render_widget(Paragraph::new(conflict_lines(state)), inner);
}

fn conflict_lines(state: &ConflictsState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if state.files.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No conflicts found",
            Style::default().fg(Color::Green),
        )));
    } else {
        for (idx, file) in state.files.iter().take(15).enumerate() {
            lines.push(selectable_item(idx == state.selected_index, file.clone()));
        }
        if state.files.len() > 15 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", state.files.len() - 15),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    let down_key = key_hint(keybindings::ModeId::Conflicts, "down", false);
    let up_key = key_hint(keybindings::ModeId::Conflicts, "up", false);
    let resolve_key = key_hint(keybindings::ModeId::Conflicts, "resolve", false);
    let exit_keys = key_hint(keybindings::ModeId::Conflicts, "exit", true);
    lines.push(Line::from(Span::styled(
        format!("{down_key}/{up_key}: navigate | {resolve_key}: resolve | {exit_keys}: exit"),
        Style::default().fg(Color::DarkGray),
    )));

    lines
}
