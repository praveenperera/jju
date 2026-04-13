use super::common::{centered_popup_area, key_hint, render_popup_shell};
use crate::cmd::jj_tui::keybindings;
use crate::cmd::jj_tui::state::ConfirmState;
use crate::cmd::jj_tui::theme;
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub(super) fn render_confirmation(frame: &mut Frame, state: &ConfirmState) {
    let area = frame.area();
    let popup_width = 80u16.min(area.width.saturating_sub(4));
    let popup_height = (7 + state.revs.len().min(10)) as u16;
    let popup_area = centered_popup_area(area, popup_width, popup_height);
    let inner = render_popup_shell(
        frame,
        popup_area,
        " Confirm ",
        Color::Red,
        theme::POPUP_BG_DELETE,
    );

    frame.render_widget(Paragraph::new(confirm_lines(state)), inner);
}

fn confirm_lines(state: &ConfirmState) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            state.message.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (idx, rev) in state.revs.iter().take(10).enumerate() {
        lines.push(Line::from(rev.to_string()));
        if idx == 9 && state.revs.len() > 10 {
            lines.push(Line::from(format!(
                "  ... and {} more",
                state.revs.len() - 10
            )));
        }
    }

    lines.push(Line::from(""));
    let yes_keys = key_hint(keybindings::ModeId::Confirm, "yes", true);
    let no_keys = key_hint(keybindings::ModeId::Confirm, "no", true);
    lines.push(Line::from(vec![
        Span::raw("  Press "),
        Span::styled(
            yes_keys,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to confirm or "),
        Span::styled(
            no_keys,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to cancel"),
    ]));

    lines
}
