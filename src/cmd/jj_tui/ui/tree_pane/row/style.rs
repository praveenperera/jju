use crate::cmd::jj_tui::theme;
use crate::cmd::jj_tui::vm::TreeRowVm;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
};

pub(super) fn prefix_color(is_source: bool) -> Color {
    if is_source {
        Color::Yellow
    } else {
        Color::Magenta
    }
}

pub(super) fn bookmark_color(is_source: bool) -> Color {
    if is_source {
        Color::Yellow
    } else {
        Color::Cyan
    }
}

pub(super) fn apply_row_style(
    vm: &TreeRowVm,
    is_source: bool,
    line: Line<'static>,
) -> Line<'static> {
    if vm.is_cursor {
        line.style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .add_modifier(Modifier::BOLD),
        )
    } else if is_source {
        line.style(Style::default().bg(theme::SOURCE_BG))
    } else if vm.is_selected {
        line.style(Style::default().bg(theme::SELECTED_BG))
    } else if vm.is_dimmed {
        line.style(Style::default().add_modifier(Modifier::DIM))
    } else {
        line
    }
}
