mod layout;
mod overlays;
mod status_bar;

use super::app::App;
use super::keybindings;
use super::preview::NodeRole;
use super::state::{DiffLineKind, DiffState};
use super::theme;
use super::tree::BookmarkInfo;
use super::vm::{Marker, TreeRowVm};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

#[cfg(test)]
use layout::PaneContent;
use layout::{pane_plan, panes_for, render_pane};

/// Format bookmarks to fit within max_width, showing "+N" for overflow
/// Diverged bookmarks are marked with * suffix
pub(crate) fn format_bookmarks_truncated(bookmarks: &[BookmarkInfo], max_width: usize) -> String {
    if bookmarks.is_empty() {
        return String::new();
    }

    let format_bookmark = |b: &BookmarkInfo| {
        if b.is_diverged {
            format!("{}*", b.name)
        } else {
            b.name.clone()
        }
    };

    if bookmarks.len() == 1 {
        return format_bookmark(&bookmarks[0]);
    }

    let mut result = String::new();

    for (i, bm) in bookmarks.iter().enumerate() {
        let bm_display = format_bookmark(bm);
        let remaining = bookmarks.len() - i - 1;
        let suffix = if remaining > 0 {
            format!(" +{}", remaining)
        } else {
            String::new()
        };
        let candidate = if result.is_empty() {
            format!("{}{}", bm_display, suffix)
        } else {
            format!("{} {}{}", result, bm_display, suffix)
        };

        if candidate.width() <= max_width {
            if remaining == 0 {
                result = candidate;
            } else {
                // add this bookmark, continue to next
                if result.is_empty() {
                    result = bm_display;
                } else {
                    result = format!("{} {}", result, bm_display);
                }
            }
        } else {
            // doesn't fit, stop here and add +N
            let overflow = bookmarks.len() - i;
            if result.is_empty() {
                return format!("{} +{}", format_bookmark(&bookmarks[0]), overflow - 1);
            }
            return format!("{} +{}", result, overflow);
        }
    }
    result
}

/// Render with pre-built view models (avoids rebuilding when caller already has them)
pub fn render_with_vms(frame: &mut Frame, app: &App, vms: &[TreeRowVm]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let panes = panes_for(chunks[0], app.split_view);
    let plan = pane_plan(app, panes.secondary.is_some());

    render_pane(frame, app, vms, panes.primary, plan.primary);
    if let (Some(area), Some(content)) = (panes.secondary, plan.secondary) {
        render_pane(frame, app, vms, area, content);
    }

    status_bar::render_status_bar(frame, app, chunks[1]);
    overlays::render_overlays(frame, app);
}

fn render_tree_with_vms(frame: &mut Frame, app: &App, area: Rect, vms: &[TreeRowVm]) {
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
    let scroll_offset = app.tree.scroll_offset;

    let mut lines: Vec<Line> = Vec::new();
    let mut line_count = 0;

    for vm in vms.iter().skip(scroll_offset) {
        if line_count >= viewport_height {
            break;
        }

        // render blank separator line between tree roots
        if vm.has_separator_before {
            lines.push(Line::default());
            line_count += 1;
            if line_count >= viewport_height {
                break;
            }
        }

        lines.push(render_row(vm));
        line_count += 1;

        // render expanded details
        if let Some(ref details) = vm.details {
            let detail_lines = render_commit_details_from_vm(vm, details);
            for detail in detail_lines {
                if line_count >= viewport_height {
                    break;
                }
                lines.push(detail);
                line_count += 1;
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Render a single row from the view model
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

    let mut spans = Vec::new();

    // determine colors based on role
    let is_source = matches!(vm.role, NodeRole::Source | NodeRole::Moving);
    let prefix_color = if is_source {
        Color::Yellow
    } else {
        Color::Magenta
    };

    let dim_color = Color::Reset;

    // add zoom marker with distinct color
    if vm.is_zoom_root {
        spans.push(Span::styled(zoom_marker, Style::default().fg(Color::Cyan)));
    }

    // add conflict marker in red
    if vm.has_conflicts {
        spans.push(Span::styled("× ", Style::default().fg(Color::Red)));
    }

    // add divergent marker in yellow
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
        spans.push(Span::raw(" "));
        let bm_color = if is_source {
            Color::Yellow
        } else {
            Color::Cyan
        };
        spans.push(Span::styled(bookmark_str, Style::default().fg(bm_color)));
    }

    spans.push(Span::styled(
        format!("  {}", vm.description),
        Style::default().fg(dim_color),
    ));

    // add markers on the right based on role/marker
    if let Some(ref marker) = vm.marker {
        match marker {
            Marker::Source => {
                spans.push(Span::styled("  ← src", Style::default().fg(Color::Yellow)));
            }
            Marker::Destination { mode_hint } => {
                let hint = mode_hint
                    .as_ref()
                    .map(|h| format!("  ← dest ({h})"))
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

    // apply styling based on state
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

/// Render commit details from view model
fn render_commit_details_from_vm(
    vm: &TreeRowVm,
    details: &super::vm::RowDetails,
) -> Vec<Line<'static>> {
    let indent = "  ".repeat(vm.visual_depth + 1);
    let dim = Style::default().fg(Color::Reset);
    let label_style = Style::default().fg(Color::Yellow);

    let change_id = format!("{}{}", vm.change_id_prefix, vm.change_id_suffix);

    let stats_str = match &details.diff_stats {
        Some(s) => format!(
            "{} file{}, +{} -{}",
            s.files_changed,
            if s.files_changed == 1 { "" } else { "s" },
            s.insertions,
            s.deletions
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
                        .map(|s| s.insertions)
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
                        .map(|s| s.deletions)
                        .unwrap_or(0)
                ),
                Style::default().fg(Color::Red),
            ),
            Span::styled(format!(" ({stats_str})"), dim),
        ]),
    ];

    // add description header
    lines.push(Line::from(vec![Span::styled(
        format!("{indent}Description:"),
        label_style,
    )]));

    // add multi-line description
    let desc_text = details.full_description.trim();
    if desc_text.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(format!("{indent}  "), label_style),
            Span::styled("(empty)", dim),
        ]));
    } else {
        for desc_line in desc_text.lines() {
            lines.push(Line::from(vec![
                Span::styled(format!("{indent}  "), label_style),
                Span::styled(desc_line.to_string(), dim),
            ]));
        }
    }

    lines
}

fn render_diff(frame: &mut Frame, state: &DiffState, area: Rect) {
    let block = Block::default()
        .title(format!(" Diff: {} ", state.rev))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let viewport_height = inner.height as usize;
    let lines: Vec<Line> = state
        .lines
        .iter()
        .skip(state.scroll_offset)
        .take(viewport_height)
        .map(|dl| {
            // apply background tint for added/removed lines
            let bg = match dl.kind {
                DiffLineKind::Added => Some(theme::DIFF_ADDED_BG),
                DiffLineKind::Removed => Some(theme::DIFF_REMOVED_BG),
                _ => None,
            };

            let spans: Vec<Span> = dl
                .spans
                .iter()
                .map(|s| {
                    let mut style = Style::default().fg(s.fg);
                    if let Some(bg_color) = bg {
                        style = style.bg(bg_color);
                    }
                    Span::styled(s.text.clone(), style)
                })
                .collect();

            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_diff_pane(frame: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Diff ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let diff_key = keybindings::display_keys_joined(
        keybindings::ModeId::Normal,
        None,
        "diff",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let hint = Paragraph::new(format!("Press {diff_key} to view diff"))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::state::{DiffLine, DiffLineKind, DiffState, ModeState, StyledSpan};
    use crate::cmd::jj_tui::tree::{TreeNode, TreeState, VisibleEntry};
    use crate::cmd::jj_tui::vm::build_tree_view;
    use ahash::HashSet;
    use ratatui::{Terminal, backend::TestBackend};
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    fn make_node(change_id: &str, depth: usize) -> TreeNode {
        TreeNode {
            change_id: change_id.to_string(),
            unique_prefix_len: 4,
            commit_id: format!("{change_id}000000"),
            unique_commit_prefix_len: 7,
            description: String::new(),
            full_description: String::new(),
            bookmarks: vec![],
            is_working_copy: false,
            has_conflicts: false,
            is_divergent: false,
            divergent_versions: vec![],
            parent_ids: vec![],
            depth,
            author_name: String::new(),
            author_email: String::new(),
            timestamp: String::new(),
        }
    }

    fn make_tree(nodes: Vec<TreeNode>) -> TreeState {
        let visible_entries: Vec<VisibleEntry> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| VisibleEntry {
                node_index: i,
                visual_depth: n.depth,
                has_separator_before: false,
            })
            .collect();
        let topology = crate::cmd::jj_tui::tree::TreeTopology::from_nodes(&nodes);

        TreeState {
            nodes,
            topology,
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            expanded_entry: None,
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }

    fn buffer_to_string(buf: &ratatui::buffer::Buffer) -> String {
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn test_split_view_renders_tree_and_diff_when_viewing_diff() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);

        let diff = DiffState {
            lines: vec![DiffLine {
                spans: vec![StyledSpan {
                    text: "diff content".to_string(),
                    fg: Color::Reset,
                }],
                kind: DiffLineKind::Context,
            }],
            scroll_offset: 0,
            rev: "aaaa".to_string(),
        };

        let app = App {
            tree,
            mode: ModeState::ViewingDiff(diff),
            should_quit: false,
            split_view: true,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_operation: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        };

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal init");

        let vms = build_tree_view(&app, 80);
        terminal
            .draw(|frame| render_with_vms(frame, &app, &vms))
            .expect("terminal draw");

        let screen = buffer_to_string(terminal.backend().buffer());
        assert!(
            screen.contains("jj tree"),
            "expected tree pane title in screen; got:\n{screen}"
        );
        assert!(
            screen.contains("Diff:"),
            "expected diff pane title in screen; got:\n{screen}"
        );
    }

    #[test]
    fn test_pane_plan_no_secondary_viewing_diff() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let diff = DiffState {
            lines: vec![],
            scroll_offset: 0,
            rev: "aaaa".to_string(),
        };
        let app = App {
            tree,
            mode: ModeState::ViewingDiff(diff),
            should_quit: false,
            split_view: false,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_operation: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        };

        let plan = pane_plan(&app, false);
        assert!(matches!(plan.primary, PaneContent::Diff(_)));
        assert!(plan.secondary.is_none());
    }

    #[test]
    fn test_pane_plan_with_secondary_viewing_diff() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let diff = DiffState {
            lines: vec![],
            scroll_offset: 0,
            rev: "aaaa".to_string(),
        };
        let app = App {
            tree,
            mode: ModeState::ViewingDiff(diff),
            should_quit: false,
            split_view: true,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_operation: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        };

        let plan = pane_plan(&app, true);
        assert!(matches!(plan.primary, PaneContent::Tree));
        assert!(matches!(plan.secondary, Some(PaneContent::Diff(_))));
    }

    #[test]
    fn test_pane_plan_with_secondary_normal_mode() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let app = App {
            tree,
            mode: ModeState::Normal,
            should_quit: false,
            split_view: true,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_operation: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        };

        let plan = pane_plan(&app, true);
        assert!(matches!(plan.primary, PaneContent::Tree));
        assert!(matches!(plan.secondary, Some(PaneContent::DiffPlaceholder)));
    }
}
