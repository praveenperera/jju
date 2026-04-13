mod current;
mod indicators;

use super::super::app::App;
use super::super::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

pub(super) fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let left = format!(
        " {}{}{}{}{}{}{}{}",
        indicators::mode_indicator(app),
        indicators::full_indicator(app),
        indicators::neighborhood_indicator(app),
        indicators::split_indicator(app),
        indicators::focus_indicator(app),
        indicators::pending_indicator(app),
        indicators::selection_indicator(app),
        current::current_info(app),
    );
    let hints = indicators::hints(app);
    let right = format!("{hints} ");

    let available = area.width as usize;
    let left_width = left.width();
    let right_width = right.width();

    let text = if left_width + right_width < available {
        let padding = available - left_width - right_width;
        format!("{left}{:padding$}{right}", "")
    } else {
        format!("{left}  {hints}")
    };

    let bar =
        Paragraph::new(text).style(Style::default().bg(theme::STATUS_BAR_BG).fg(Color::White));

    frame.render_widget(bar, area);
}
