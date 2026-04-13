use crate::cmd::jj_tui::state::{MessageKind, StatusMessage};
use crate::cmd::jj_tui::theme;
use ratatui::{
    Frame,
    style::{Color, Style},
};
use tui_popup::Popup;

pub(super) fn render_toast(frame: &mut Frame, msg: &StatusMessage) {
    let color = match msg.kind {
        MessageKind::Success => Color::Green,
        MessageKind::Warning => Color::Yellow,
        MessageKind::Error => Color::Red,
    };

    frame.render_widget(
        Popup::new(msg.text.clone()).style(Style::default().fg(color).bg(theme::TOAST_BG)),
        frame.area(),
    );
}
