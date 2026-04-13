use super::bookmarks::format_bookmarks_truncated;
use crate::cmd::jj_tui::preview::NodeRole;
use crate::cmd::jj_tui::theme;
use crate::cmd::jj_tui::vm::{Marker, TreeRowVm};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub(super) fn render_row(vm: &TreeRowVm) -> Line<'static> {
    let is_source = matches!(vm.role, NodeRole::Source | NodeRole::Moving);
    let prefix_color = if is_source {
        Color::Yellow
    } else {
        Color::Magenta
    };
    let mut spans = indicator_spans(vm);

    spans.extend([
        Span::raw(format!(
            "{}{}{}{}(",
            "  ".repeat(vm.visual_depth),
            connector(vm.visual_depth),
            selection_marker(vm.is_selected),
            working_copy_marker(vm.is_working_copy),
        )),
        Span::styled(
            vm.change_id_prefix.clone(),
            Style::default().fg(prefix_color),
        ),
        Span::styled(
            vm.change_id_suffix.clone(),
            Style::default().add_modifier(Modifier::DIM),
        ),
        Span::raw(")"),
    ]);

    if !vm.bookmarks.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format_bookmarks_truncated(&vm.bookmarks, 30),
            Style::default().fg(bookmark_color(is_source)),
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
        spans.push(render_marker(marker));
    }

    let mut line = Line::from(spans);
    if vm.is_cursor {
        line = line.style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .add_modifier(Modifier::BOLD),
        );
    } else if is_source {
        line = line.style(Style::default().bg(theme::SOURCE_BG));
    } else if vm.is_selected {
        line = line.style(Style::default().bg(theme::SELECTED_BG));
    } else if vm.is_dimmed {
        line = line.style(Style::default().add_modifier(Modifier::DIM));
    }

    line
}

fn indicator_spans(vm: &TreeRowVm) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    if vm.is_zoom_root {
        spans.push(Span::styled("◉ ", Style::default().fg(Color::Cyan)));
    }
    if vm.has_conflicts {
        spans.push(Span::styled("× ", Style::default().fg(Color::Red)));
    }
    if vm.is_divergent {
        spans.push(Span::styled("?? ", Style::default().fg(Color::Yellow)));
    }

    spans
}

fn connector(visual_depth: usize) -> &'static str {
    if visual_depth > 0 { "├── " } else { "" }
}

fn selection_marker(is_selected: bool) -> &'static str {
    if is_selected { "[x] " } else { "" }
}

fn working_copy_marker(is_working_copy: bool) -> &'static str {
    if is_working_copy { "@ " } else { "" }
}

fn bookmark_color(is_source: bool) -> Color {
    if is_source {
        Color::Yellow
    } else {
        Color::Cyan
    }
}

fn render_marker(marker: &Marker) -> Span<'static> {
    match marker {
        Marker::Source => Span::styled("  ← src", Style::default().fg(Color::Yellow)),
        Marker::Destination { mode_hint } => Span::styled(
            mode_hint
                .as_ref()
                .map(|mode_hint| format!("  ← dest ({mode_hint})"))
                .unwrap_or_else(|| "  ← dest".to_string()),
            Style::default().fg(Color::Cyan),
        ),
        Marker::Moving => Span::styled("  ↳", Style::default().fg(Color::Yellow)),
        Marker::Bookmark => Span::styled("  ← bm", Style::default().fg(Color::Yellow)),
    }
}
