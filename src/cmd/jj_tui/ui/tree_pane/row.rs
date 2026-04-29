use super::bookmarks::format_bookmarks_truncated;
use crate::cmd::jj_tui::preview::NodeRole;
use crate::cmd::jj_tui::theme;
use crate::cmd::jj_tui::vm::{InlineRowBadge, Marker, TreeRowVm};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub(super) fn render_row(vm: &TreeRowVm) -> Line<'static> {
    let is_source = matches!(vm.role, NodeRole::Source | NodeRole::Moving);
    let mut spans = indicator_spans(vm);

    spans.extend([
        prefix_fragment(vm),
        Span::styled(
            vm.change_id_prefix.clone(),
            Style::default().fg(prefix_color(is_source)),
        ),
        suffix_fragment(vm),
        Span::raw(")"),
    ]);

    if !vm.bookmarks.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format_bookmarks_truncated(&vm.bookmarks, 30),
            Style::default().fg(bookmark_color(is_source)),
        ));
    }

    spans.extend(inline_badge_spans(vm));
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

    apply_row_style(vm, is_source, Line::from(spans))
}

fn inline_badge_spans(vm: &TreeRowVm) -> Vec<Span<'static>> {
    match vm.inline_badge.as_ref() {
        Some(InlineRowBadge::DiffStats(stats)) => vec![
            Span::raw("  "),
            Span::styled(
                format!("+{}", stats.insertions),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" "),
            Span::styled(
                format!("-{}", stats.deletions),
                Style::default().fg(Color::Red),
            ),
        ],
        Some(InlineRowBadge::EmptyRevision) => vec![
            Span::raw("  "),
            Span::styled("∅", Style::default().fg(Color::Yellow)),
        ],
        None => Vec::new(),
    }
}

fn prefix_fragment(vm: &TreeRowVm) -> Span<'static> {
    Span::raw(format!(
        "{}{}{}{}(",
        "  ".repeat(vm.visual_depth),
        connector(vm.visual_depth),
        selection_marker(vm.is_selected),
        working_copy_marker(vm.is_working_copy),
    ))
}

fn suffix_fragment(vm: &TreeRowVm) -> Span<'static> {
    Span::styled(
        vm.change_id_suffix.clone(),
        Style::default().add_modifier(Modifier::DIM),
    )
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

fn connector(visual_depth: usize) -> &'static str {
    if visual_depth > 0 { "├── " } else { "" }
}

fn selection_marker(is_selected: bool) -> &'static str {
    if is_selected { "[x] " } else { "" }
}

fn working_copy_marker(is_working_copy: bool) -> &'static str {
    if is_working_copy { "@ " } else { "" }
}

fn prefix_color(is_source: bool) -> Color {
    if is_source {
        Color::Yellow
    } else {
        Color::Magenta
    }
}

fn bookmark_color(is_source: bool) -> Color {
    if is_source {
        Color::Yellow
    } else {
        Color::Cyan
    }
}

fn apply_row_style(vm: &TreeRowVm, is_source: bool, line: Line<'static>) -> Line<'static> {
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

#[cfg(test)]
mod tests {
    use super::render_row;
    use crate::cmd::jj_tui::state::DiffStats;
    use crate::cmd::jj_tui::vm::{InlineRowBadge, TreeRowVm};

    fn row_text(vm: &TreeRowVm) -> String {
        render_row(vm)
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
    }

    fn make_vm() -> TreeRowVm {
        TreeRowVm {
            visual_depth: 0,
            role: crate::cmd::jj_tui::preview::NodeRole::Normal,
            is_cursor: false,
            is_selected: false,
            is_dimmed: false,
            is_zoom_root: false,
            is_working_copy: false,
            has_conflicts: false,
            is_divergent: false,
            change_id_prefix: "abcd".to_string(),
            change_id_suffix: String::new(),
            bookmarks: vec![],
            description: "desc".to_string(),
            inline_badge: None,
            is_neighborhood_preview: false,
            neighborhood_hidden_count: 0,
            marker: None,
            details: None,
            height: 1,
            has_separator_before: false,
        }
    }

    #[test]
    fn renders_empty_marker_for_empty_revision() {
        let mut vm = make_vm();
        vm.inline_badge = Some(InlineRowBadge::EmptyRevision);

        let row = row_text(&vm);

        assert!(row.contains("∅"));
    }

    #[test]
    fn renders_inline_diff_stats_for_working_copy_or_cursor_rows() {
        let mut vm = make_vm();
        vm.inline_badge = Some(InlineRowBadge::DiffStats(DiffStats {
            files_changed: 1,
            insertions: 3,
            deletions: 2,
        }));

        let row = row_text(&vm);

        assert!(row.contains("+3 -2"));
    }

    #[test]
    fn suppresses_empty_marker_when_inline_diff_stats_are_shown() {
        let mut vm = make_vm();
        vm.inline_badge = Some(InlineRowBadge::DiffStats(DiffStats {
            files_changed: 0,
            insertions: 0,
            deletions: 0,
        }));

        let row = row_text(&vm);

        assert!(row.contains("+0 -0"));
        assert!(!row.contains("∅"));
    }
}
