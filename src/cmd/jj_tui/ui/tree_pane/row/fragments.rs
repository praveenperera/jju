use crate::cmd::jj_tui::vm::{Marker, TreeRowVm};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

pub(super) fn prefix_fragment(vm: &TreeRowVm) -> Span<'static> {
    Span::raw(format!(
        "{}{}{}{}(",
        "  ".repeat(vm.visual_depth),
        connector(vm.visual_depth),
        selection_marker(vm.is_selected),
        working_copy_marker(vm.is_working_copy),
    ))
}

pub(super) fn suffix_fragment(vm: &TreeRowVm) -> Span<'static> {
    Span::styled(
        vm.change_id_suffix.clone(),
        Style::default().add_modifier(Modifier::DIM),
    )
}

pub(super) fn indicator_spans(vm: &TreeRowVm) -> Vec<Span<'static>> {
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

pub(super) fn render_marker(marker: &Marker) -> Span<'static> {
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

fn connector(visual_depth: usize) -> &'static str {
    if visual_depth > 0 { "├── " } else { "" }
}

fn selection_marker(is_selected: bool) -> &'static str {
    if is_selected { "[x] " } else { "" }
}

fn working_copy_marker(is_working_copy: bool) -> &'static str {
    if is_working_copy { "@ " } else { "" }
}
