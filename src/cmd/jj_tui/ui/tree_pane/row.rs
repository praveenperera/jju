mod fragments;
mod style;

use super::bookmarks::format_bookmarks_truncated;
use crate::cmd::jj_tui::preview::NodeRole;
use crate::cmd::jj_tui::vm::TreeRowVm;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub(super) fn render_row(vm: &TreeRowVm) -> Line<'static> {
    let is_source = matches!(vm.role, NodeRole::Source | NodeRole::Moving);
    let mut spans = fragments::indicator_spans(vm);

    spans.extend([
        fragments::prefix_fragment(vm),
        Span::styled(
            vm.change_id_prefix.clone(),
            Style::default().fg(style::prefix_color(is_source)),
        ),
        fragments::suffix_fragment(vm),
        Span::raw(")"),
    ]);

    if !vm.bookmarks.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format_bookmarks_truncated(&vm.bookmarks, 30),
            Style::default().fg(style::bookmark_color(is_source)),
        ));
    }

    spans.push(Span::styled(
        format!("  {}", vm.description),
        Style::default().fg(Color::Reset),
    ));

    if vm.is_neighborhood_preview && vm.neighborhood_hidden_count > 0 {
        spans.push(Span::styled(
            format!("  [+{} more, Enter]", vm.neighborhood_hidden_count),
            Style::default().fg(Color::Cyan),
        ));
    }

    if let Some(marker) = vm.marker.as_ref() {
        spans.push(fragments::render_marker(marker));
    }

    style::apply_row_style(vm, is_source, Line::from(spans))
}
