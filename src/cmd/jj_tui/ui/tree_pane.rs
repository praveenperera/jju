use crate::cmd::jj_tui::app::App;
use crate::cmd::jj_tui::preview::NodeRole;
use crate::cmd::jj_tui::theme;
use crate::cmd::jj_tui::tree::BookmarkInfo;
use crate::cmd::jj_tui::vm::{Marker, RowDetails, TreeRowVm};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

/// Format bookmarks to fit within max_width, showing "+N" for overflow
/// Diverged bookmarks are marked with * suffix
pub(crate) fn format_bookmarks_truncated(bookmarks: &[BookmarkInfo], max_width: usize) -> String {
    if bookmarks.is_empty() {
        return String::new();
    }

    let format_bookmark = |bookmark: &BookmarkInfo| {
        if bookmark.is_diverged {
            format!("{}*", bookmark.name)
        } else {
            bookmark.name.clone()
        }
    };

    if bookmarks.len() == 1 {
        return format_bookmark(&bookmarks[0]);
    }

    let mut result = String::new();

    for (index, bookmark) in bookmarks.iter().enumerate() {
        let bookmark_display = format_bookmark(bookmark);
        let remaining = bookmarks.len() - index - 1;
        let suffix = if remaining > 0 {
            format!(" +{}", remaining)
        } else {
            String::new()
        };
        let candidate = if result.is_empty() {
            format!("{bookmark_display}{suffix}")
        } else {
            format!("{result} {bookmark_display}{suffix}")
        };

        if candidate.width() <= max_width {
            if remaining == 0 {
                result = candidate;
            } else if result.is_empty() {
                result = bookmark_display;
            } else {
                result = format!("{result} {bookmark_display}");
            }
            continue;
        }

        let overflow = bookmarks.len() - index;
        if result.is_empty() {
            return format!("{} +{}", format_bookmark(&bookmarks[0]), overflow - 1);
        }
        return format!("{result} +{overflow}");
    }

    result
}

pub(crate) fn render_tree_with_vms(frame: &mut Frame, app: &App, area: Rect, vms: &[TreeRowVm]) {
    let block = Block::default()
        .title(" jj tree ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tree.visible_count() == 0 {
        let empty = Paragraph::new("No commits found").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    let viewport_height = inner.height as usize;
    let scroll_offset = app.tree.view.scroll_offset;

    let mut lines: Vec<Line> = Vec::new();
    let mut line_count = 0;

    for vm in vms.iter().skip(scroll_offset) {
        if line_count >= viewport_height {
            break;
        }

        if vm.has_separator_before {
            lines.push(Line::default());
            line_count += 1;
            if line_count >= viewport_height {
                break;
            }
        }

        lines.push(render_row(vm));
        line_count += 1;

        if let Some(details) = vm.details.as_ref() {
            for detail_line in render_commit_details_from_vm(vm, details) {
                if line_count >= viewport_height {
                    break;
                }
                lines.push(detail_line);
                line_count += 1;
            }
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_row(vm: &TreeRowVm) -> Line<'static> {
    let indent = "  ".repeat(vm.visual_depth);
    let connector = if vm.visual_depth > 0 {
        "├── "
    } else {
        ""
    };
    let at_marker = if vm.is_working_copy { "@ " } else { "" };
    let selection_marker = if vm.is_selected { "[x] " } else { "" };
    let zoom_marker = if vm.is_zoom_root { "◉ " } else { "" };

    let is_source = matches!(vm.role, NodeRole::Source | NodeRole::Moving);
    let prefix_color = if is_source {
        Color::Yellow
    } else {
        Color::Magenta
    };

    let mut spans = Vec::new();

    if vm.is_zoom_root {
        spans.push(Span::styled(zoom_marker, Style::default().fg(Color::Cyan)));
    }
    if vm.has_conflicts {
        spans.push(Span::styled("× ", Style::default().fg(Color::Red)));
    }
    if vm.is_divergent {
        spans.push(Span::styled("?? ", Style::default().fg(Color::Yellow)));
    }

    spans.extend([
        Span::raw(format!("{indent}{connector}{selection_marker}{at_marker}(")),
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
        let bookmark_str = format_bookmarks_truncated(&vm.bookmarks, 30);
        let bookmark_color = if is_source {
            Color::Yellow
        } else {
            Color::Cyan
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            bookmark_str,
            Style::default().fg(bookmark_color),
        ));
    }

    spans.push(Span::styled(
        format!("  {}", vm.description),
        Style::default().fg(Color::Reset),
    ));

    if let Some(marker) = vm.marker.as_ref() {
        match marker {
            Marker::Source => {
                spans.push(Span::styled("  ← src", Style::default().fg(Color::Yellow)));
            }
            Marker::Destination { mode_hint } => {
                let hint = mode_hint
                    .as_ref()
                    .map(|mode_hint| format!("  ← dest ({mode_hint})"))
                    .unwrap_or_else(|| "  ← dest".to_string());
                spans.push(Span::styled(hint, Style::default().fg(Color::Cyan)));
            }
            Marker::Moving => {
                spans.push(Span::styled("  ↳", Style::default().fg(Color::Yellow)));
            }
            Marker::Bookmark => {
                spans.push(Span::styled("  ← bm", Style::default().fg(Color::Yellow)));
            }
        }
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

fn render_commit_details_from_vm(vm: &TreeRowVm, details: &RowDetails) -> Vec<Line<'static>> {
    let indent = "  ".repeat(vm.visual_depth + 1);
    let dim = Style::default().fg(Color::Reset);
    let label_style = Style::default().fg(Color::Yellow);
    let change_id = format!("{}{}", vm.change_id_prefix, vm.change_id_suffix);
    let stats_str = match details.diff_stats.as_ref() {
        Some(stats) => format!(
            "{} file{}, +{} -{}",
            stats.files_changed,
            if stats.files_changed == 1 { "" } else { "s" },
            stats.insertions,
            stats.deletions
        ),
        None => "loading...".to_string(),
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{indent}Change ID: "), label_style),
            Span::styled(change_id, dim),
        ]),
        Line::from(vec![
            Span::styled(format!("{indent}Commit: "), label_style),
            Span::styled(
                details.commit_id_prefix.clone(),
                Style::default().fg(Color::Blue),
            ),
            Span::styled(
                details.commit_id_suffix.clone(),
                Style::default().add_modifier(Modifier::DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("{indent}Author: "), label_style),
            Span::styled(details.author.clone(), dim),
        ]),
        Line::from(vec![
            Span::styled(format!("{indent}Date: "), label_style),
            Span::styled(details.timestamp.clone(), dim),
        ]),
        Line::from(vec![
            Span::styled(format!("{indent}Changes: "), label_style),
            Span::styled(
                format!(
                    "+{}",
                    details
                        .diff_stats
                        .as_ref()
                        .map(|stats| stats.insertions)
                        .unwrap_or(0)
                ),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" "),
            Span::styled(
                format!(
                    "-{}",
                    details
                        .diff_stats
                        .as_ref()
                        .map(|stats| stats.deletions)
                        .unwrap_or(0)
                ),
                Style::default().fg(Color::Red),
            ),
            Span::styled(format!(" ({stats_str})"), dim),
        ]),
    ];

    lines.push(Line::from(vec![Span::styled(
        format!("{indent}Description:"),
        label_style,
    )]));

    let description = details.full_description.trim();
    if description.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(format!("{indent}  "), label_style),
            Span::styled("(empty)", dim),
        ]));
        return lines;
    }

    for line in description.lines() {
        lines.push(Line::from(vec![
            Span::styled(format!("{indent}  "), label_style),
            Span::styled(line.to_string(), dim),
        ]));
    }

    lines
}
