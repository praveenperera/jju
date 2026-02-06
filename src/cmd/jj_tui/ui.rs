use super::app::App;
use super::keybindings;
use super::preview::NodeRole;
use super::state::{
    BookmarkInputState, BookmarkPickerState, BookmarkSelectAction, BookmarkSelectState,
    ConfirmState, ConflictsState, DiffLineKind, DiffState, MessageKind, ModeState, PushSelectState,
    RebaseType, StatusMessage,
};
use super::theme;
use super::tree::BookmarkInfo;
use super::vm::{Marker, TreeRowVm, build_tree_view};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use tui_popup::Popup;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy)]
struct Panes {
    primary: Rect,
    secondary: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
enum PaneContent<'a> {
    Tree,
    Diff(&'a DiffState),
    DiffPlaceholder,
}

#[derive(Debug, Clone, Copy)]
struct PanePlan<'a> {
    primary: PaneContent<'a>,
    secondary: Option<PaneContent<'a>>,
}

fn panes_for(area: Rect, split_view: bool) -> Panes {
    if split_view {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        Panes {
            primary: split[0],
            secondary: Some(split[1]),
        }
    } else {
        Panes {
            primary: area,
            secondary: None,
        }
    }
}

fn pane_plan<'a>(app: &'a App, has_secondary: bool) -> PanePlan<'a> {
    if has_secondary {
        match &app.mode {
            ModeState::ViewingDiff(state) => PanePlan {
                primary: PaneContent::Tree,
                secondary: Some(PaneContent::Diff(state)),
            },
            _ => PanePlan {
                primary: PaneContent::Tree,
                secondary: Some(PaneContent::DiffPlaceholder),
            },
        }
    } else {
        match &app.mode {
            ModeState::ViewingDiff(state) => PanePlan {
                primary: PaneContent::Diff(state),
                secondary: None,
            },
            _ => PanePlan {
                primary: PaneContent::Tree,
                secondary: None,
            },
        }
    }
}

fn render_pane(
    frame: &mut Frame,
    app: &App,
    vms: &[TreeRowVm],
    area: Rect,
    content: PaneContent<'_>,
) {
    match content {
        PaneContent::Tree => render_tree_with_vms(frame, app, area, vms),
        PaneContent::Diff(state) => render_diff(frame, state, area),
        PaneContent::DiffPlaceholder => render_diff_pane(frame, app, area),
    }
}

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

#[allow(dead_code)]
pub fn render(frame: &mut Frame, app: &App) {
    let vms = build_tree_view(app, frame.area().width as usize);
    render_with_vms(frame, app, &vms);
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

    render_status_bar(frame, app, chunks[1]);

    // render overlays
    if app.mode.is_help() {
        render_help(frame);
    }

    if let ModeState::Confirming(ref state) = app.mode {
        render_confirmation(frame, state);
    }

    if let ModeState::BookmarkInput(ref state) = app.mode {
        render_bookmark_input(frame, state);
    }

    if let ModeState::BookmarkSelect(ref state) = app.mode {
        render_bookmark_select(frame, state);
    }

    if let ModeState::BookmarkPicker(ref state) = app.mode {
        render_bookmark_picker(frame, state);
    }

    if let ModeState::PushSelect(ref state) = app.mode {
        render_push_select(frame, state);
    }

    if let ModeState::Conflicts(ref state) = app.mode {
        render_conflicts_panel(frame, state);
    }

    // render prefix key popup when waiting for second key in sequence
    if let Some(pending) = app.pending_key {
        render_prefix_key_popup(frame, keybindings::mode_id_from_state(&app.mode), pending);
    }

    // render toast notification last (on top of everything)
    if let Some(ref msg) = app.status_message
        && std::time::Instant::now() < msg.expires
    {
        render_toast(frame, msg);
    }
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

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mode_indicator = match &app.mode {
        ModeState::Normal => "NORMAL",
        ModeState::Help => "HELP",
        ModeState::ViewingDiff(_) => "DIFF",
        ModeState::Confirming(_) => "CONFIRM",
        ModeState::Selecting => "SELECT",
        ModeState::Rebasing(state) => {
            if state.rebase_type == RebaseType::Single {
                "REBASE -r"
            } else {
                "REBASE -s"
            }
        }
        ModeState::MovingBookmark(_) => "MOVE BOOKMARK",
        ModeState::BookmarkInput(_) => "BOOKMARK",
        ModeState::BookmarkSelect(_) => "SELECT BM",
        ModeState::BookmarkPicker(_) => "PICK BM",
        ModeState::PushSelect(_) => "PUSH SELECT",
        ModeState::Squashing(_) => "SQUASH",
        ModeState::Conflicts(_) => "CONFLICTS",
    };

    let full_indicator = if app.tree.full_mode { " [FULL]" } else { "" };
    let split_indicator = if app.split_view { " [SPLIT]" } else { "" };

    // show zoom indicator with depth and focused node info
    let focus_indicator = if app.tree.is_focused() {
        let depth = app.tree.focus_depth();
        let focused_name = app
            .tree
            .focused_node()
            .map(|n| {
                if !n.bookmarks.is_empty() {
                    n.bookmark_names().first().cloned().unwrap_or_default()
                } else {
                    n.change_id.chars().take(8).collect::<String>()
                }
            })
            .unwrap_or_default();
        format!(" [ZOOM:{depth}→{focused_name}]")
    } else {
        String::new()
    };

    // show pending key when waiting for second key in sequence
    let pending_indicator = match app.pending_key {
        Some(p) if keybindings::is_known_prefix(p) => format!(" {p}-"),
        _ => String::new(),
    };

    // show selection count when there are selected items
    let selection_indicator = if !app.tree.selected.is_empty() {
        format!(" [{}sel]", app.tree.selected.len())
    } else {
        String::new()
    };

    // in rebase mode, show source→dest instead of current node
    let current_info = if let ModeState::Rebasing(state) = &app.mode {
        let dest_name = app
            .tree
            .visible_entries
            .get(state.dest_cursor)
            .map(|e| {
                let node = &app.tree.nodes[e.node_index];
                if node.bookmarks.is_empty() {
                    node.change_id.chars().take(8).collect::<String>()
                } else {
                    node.bookmark_names().join(" ")
                }
            })
            .unwrap_or_else(|| "?".to_string());
        let src_short: String = state.source_rev.chars().take(8).collect();
        format!(" | {src_short}→{dest_name}")
    } else if let ModeState::MovingBookmark(state) = &app.mode {
        let dest_name = app
            .tree
            .visible_entries
            .get(state.dest_cursor)
            .map(|e| {
                let node = &app.tree.nodes[e.node_index];
                node.change_id.chars().take(8).collect::<String>()
            })
            .unwrap_or_else(|| "?".to_string());
        let bm_name: String = state.bookmark_name.chars().take(12).collect();
        format!(" | {bm_name}→{dest_name}")
    } else if let ModeState::Squashing(state) = &app.mode {
        let dest_name = app
            .tree
            .visible_entries
            .get(state.dest_cursor)
            .map(|e| {
                let node = &app.tree.nodes[e.node_index];
                if node.bookmarks.is_empty() {
                    node.change_id.chars().take(8).collect::<String>()
                } else {
                    node.bookmark_names().join(" ")
                }
            })
            .unwrap_or_else(|| "?".to_string());
        let src_short: String = state.source_rev.chars().take(8).collect();
        format!(" | {src_short}→{dest_name}")
    } else {
        app.tree
            .current_node()
            .map(|n| {
                let name = if n.bookmarks.is_empty() {
                    n.change_id.clone()
                } else {
                    n.bookmark_names().join(" ")
                };
                format!(" | {name}")
            })
            .unwrap_or_default()
    };

    let mode_id = keybindings::mode_id_from_state(&app.mode);
    let rebase_allow_branches = match &app.mode {
        ModeState::Rebasing(state) => Some(state.allow_branches),
        _ => None,
    };
    let hints = keybindings::status_bar_hints(&keybindings::StatusHintContext {
        mode: mode_id,
        has_selection: !app.tree.selected.is_empty(),
        has_focus: app.tree.is_focused(),
        current_has_bookmark: app.current_has_bookmark(),
        rebase_allow_branches,
    });

    let left = format!(
        " {mode_indicator}{full_indicator}{split_indicator}{focus_indicator}{pending_indicator}{selection_indicator}{current_info}"
    );
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

fn render_help(frame: &mut Frame) {
    use unicode_width::UnicodeWidthStr;

    let view = keybindings::build_help_view();
    let key_col_width = view
        .iter()
        .flat_map(|s| s.items.iter().map(|i| i.keys.width()))
        .max()
        .unwrap_or(9)
        .max(9);

    let mut help_text: Vec<Line> = Vec::new();
    for (section_idx, section) in view.iter().enumerate() {
        if section_idx > 0 {
            help_text.push(Line::from(""));
        }
        help_text.push(Line::from(Span::styled(
            section.title,
            Style::default().add_modifier(Modifier::BOLD),
        )));

        for item in &section.items {
            let pad_len = key_col_width.saturating_sub(item.keys.width());
            let pad = " ".repeat(pad_len);
            help_text.push(Line::from(format!(
                "  {}{}  {}",
                item.keys, pad, item.description
            )));
        }
    }

    // calculate dimensions from content
    let area = frame.area();
    let content_width = help_text
        .iter()
        .map(|line| line.spans.iter().map(|s| s.content.width()).sum::<usize>())
        .max()
        .unwrap_or(0) as u16;
    let content_height = help_text.len() as u16;

    // add padding for borders (2) and some margin (2)
    let popup_width = (content_width + 4).min(area.width.saturating_sub(4));
    let popup_height = (content_height + 2).min(area.height.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(theme::POPUP_BG));

    frame.render_widget(help, popup_area);
}

fn render_confirmation(frame: &mut Frame, state: &ConfirmState) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));

    // calculate height based on content
    let rev_count = state.revs.len();
    let popup_height = (7 + rev_count.min(5)) as u16; // message + revs (max 5) + padding + buttons

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(
        block.style(Style::default().bg(theme::POPUP_BG_DELETE)),
        popup_area,
    );

    let mut lines = vec![
        Line::from(Span::styled(
            state.message.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // show affected revisions (up to 5)
    for (i, rev) in state.revs.iter().take(5).enumerate() {
        lines.push(Line::from(format!("  {rev}")));
        if i == 4 && state.revs.len() > 5 {
            lines.push(Line::from(format!(
                "  ... and {} more",
                state.revs.len() - 5
            )));
        }
    }

    lines.push(Line::from(""));
    let yes_keys = keybindings::display_keys_joined(
        keybindings::ModeId::Confirm,
        None,
        "yes",
        true,
        keybindings::KeyFormat::Space,
        "/",
    );
    let no_keys = keybindings::display_keys_joined(
        keybindings::ModeId::Confirm,
        None,
        "no",
        true,
        keybindings::KeyFormat::Space,
        "/",
    );
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

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
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
    use ahash::{HashMap, HashSet};
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
            })
            .collect();

        TreeState {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            expanded_entry: None,
            children_map: HashMap::default(),
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

fn render_bookmark_input(frame: &mut Frame, state: &BookmarkInputState) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 7u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let title = if state.deleting {
        " Delete Bookmark "
    } else {
        " Create Bookmark "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.deleting {
            Color::Red
        } else {
            Color::Cyan
        }));

    let inner = block.inner(popup_area);
    let bg_color = if state.deleting {
        theme::POPUP_BG_DELETE
    } else {
        theme::POPUP_BG
    };
    frame.render_widget(block.style(Style::default().bg(bg_color)), popup_area);

    // render text with cursor
    let before = &state.name[..state.cursor];
    let cursor_char = state
        .name
        .get(state.cursor..)
        .and_then(|s| s.chars().next());
    let after = if let Some(c) = cursor_char {
        &state.name[state.cursor + c.len_utf8()..]
    } else {
        ""
    };
    let cursor_display = cursor_char.unwrap_or(' ');

    let input_line = Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Color::Yellow)),
        Span::raw(before.to_string()),
        Span::styled(
            cursor_display.to_string(),
            Style::default().bg(Color::White).fg(Color::Black),
        ),
        Span::raw(after.to_string()),
    ]);

    let target_short: String = state.target_rev.chars().take(8).collect();
    let target_line = Line::from(vec![
        Span::styled("At: ", Style::default().fg(Color::Yellow)),
        Span::styled(target_short, Style::default().fg(Color::DarkGray)),
    ]);

    let confirm_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkInput,
        None,
        "confirm",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let cancel_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkInput,
        None,
        "cancel",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let help_text = if state.deleting {
        format!("{confirm_key}: delete  |  {cancel_key}: cancel")
    } else {
        format!("{confirm_key}: create  |  {cancel_key}: cancel")
    };

    let lines = vec![
        input_line,
        Line::from(""),
        target_line,
        Line::from(""),
        Line::from(Span::styled(
            help_text,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_bookmark_select(frame: &mut Frame, state: &BookmarkSelectState) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = (6 + state.bookmarks.len().min(10)) as u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let (title, border_color, bg_color) = match state.action {
        BookmarkSelectAction::Move => (" Select Bookmark to Move ", Color::Cyan, theme::POPUP_BG),
        BookmarkSelectAction::Delete => (
            " Select Bookmark to Delete ",
            Color::Red,
            theme::POPUP_BG_DELETE,
        ),
        BookmarkSelectAction::CreatePR => {
            (" PR from Bookmark ", Color::Green, theme::POPUP_BG)
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(popup_area);
    frame.render_widget(block.style(Style::default().bg(bg_color)), popup_area);

    let mut lines: Vec<Line> = Vec::new();

    // show revision context
    let rev_short: String = state.target_rev.chars().take(8).collect();
    lines.push(Line::from(vec![
        Span::styled("At: ", Style::default().fg(Color::Yellow)),
        Span::styled(rev_short, Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    for (i, bookmark) in state.bookmarks.iter().enumerate() {
        let marker = if i == state.selected_index {
            "> "
        } else {
            "  "
        };
        let style = if i == state.selected_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(
            format!("{marker}{bookmark}"),
            style,
        )));
    }

    lines.push(Line::from(""));
    let down_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkSelect,
        None,
        "down",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let up_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkSelect,
        None,
        "up",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let select_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkSelect,
        None,
        "select",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let cancel_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkSelect,
        None,
        "cancel",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    lines.push(Line::from(Span::styled(
        format!("{down_key}/{up_key}: navigate | {select_key}: select | {cancel_key}: cancel"),
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_bookmark_picker(frame: &mut Frame, state: &BookmarkPickerState) {
    let area = frame.area();
    let filtered = state.filtered_bookmarks();
    let list_height = filtered.len().min(10);
    let popup_height = (8 + list_height) as u16;
    let popup_width = 60u16.min(area.width.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let (title, border_color, bg_color) = match state.action {
        BookmarkSelectAction::Move => (" Move Bookmark Here ", Color::Cyan, theme::POPUP_BG),
        BookmarkSelectAction::Delete => (" Delete Bookmark ", Color::Red, theme::POPUP_BG_DELETE),
        BookmarkSelectAction::CreatePR => {
            (" PR from Bookmark ", Color::Green, theme::POPUP_BG)
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(popup_area);
    frame.render_widget(block.style(Style::default().bg(bg_color)), popup_area);

    let mut lines: Vec<Line> = Vec::new();

    // show revision context (only for move action)
    if matches!(state.action, BookmarkSelectAction::Move) {
        let rev_short: String = state.target_rev.chars().take(8).collect();
        lines.push(Line::from(vec![
            Span::styled("Move to: ", Style::default().fg(Color::Yellow)),
            Span::styled(rev_short, Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }

    // show filter input
    let filter_display = if state.filter.is_empty() {
        "type to filter...".to_string()
    } else {
        state.filter.clone()
    };
    let filter_style = if state.filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(vec![
        Span::styled("Filter: ", Style::default().fg(Color::Green)),
        Span::styled(filter_display, filter_style),
        Span::styled("█", Style::default().fg(Color::Cyan)), // cursor
    ]));
    lines.push(Line::from(""));

    // show filtered bookmarks
    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no matching bookmarks)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, bookmark) in filtered.iter().take(10).enumerate() {
            let marker = if i == state.selected_index {
                "> "
            } else {
                "  "
            };
            let style = if i == state.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!("{marker}{bookmark}"),
                style,
            )));
        }
        // show ellipsis if there are more items
        if filtered.len() > 10 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", filtered.len() - 10),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    let up_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkPicker,
        None,
        "up",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let down_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkPicker,
        None,
        "down",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let confirm_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkPicker,
        None,
        "confirm",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let cancel_key = keybindings::display_keys_joined(
        keybindings::ModeId::BookmarkPicker,
        None,
        "cancel",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let footer = match state.action {
        BookmarkSelectAction::Move => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: move (or move away if already here) | {cancel_key}: cancel"
        ),
        BookmarkSelectAction::Delete => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: delete | {cancel_key}: cancel"
        ),
        BookmarkSelectAction::CreatePR => format!(
            "type: filter | {up_key}/{down_key}: navigate | {confirm_key}: PR | {cancel_key}: cancel"
        ),
    };
    lines.push(Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_push_select(frame: &mut Frame, state: &PushSelectState) {
    let area = frame.area();
    let filtered = state.filtered_bookmarks();
    let list_height = filtered.len().clamp(1, 10);
    let popup_height = (8 + list_height) as u16;
    let popup_width = 50u16.min(area.width.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Push Bookmarks ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(
        block.style(Style::default().bg(theme::POPUP_BG)),
        popup_area,
    );

    let mut lines: Vec<Line> = Vec::new();

    // show selection count
    let selected_count = state.selected_filtered_count();
    let total_filtered = filtered.len();
    lines.push(Line::from(Span::styled(
        format!(
            "Select bookmarks to push: {}/{} selected",
            selected_count, total_filtered
        ),
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(""));

    // show filter input
    let filter_display = if state.filter.is_empty() {
        "type to filter...".to_string()
    } else {
        state.filter.clone()
    };
    let filter_style = if state.filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(vec![
        Span::styled("Filter: ", Style::default().fg(Color::Green)),
        Span::styled(filter_display, filter_style),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(""));

    // show bookmarks with checkboxes
    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no matching bookmarks)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (display_idx, (original_idx, bookmark)) in filtered.iter().take(10).enumerate() {
            let is_cursor = display_idx == state.cursor_index;
            let is_selected = state.selected.contains(original_idx);

            let marker = if is_cursor { "> " } else { "  " };
            let checkbox = if is_selected { "[x] " } else { "[ ] " };

            let style = if is_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(Span::styled(
                format!("{marker}{checkbox}{bookmark}"),
                style,
            )));
        }

        if filtered.len() > 10 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", filtered.len() - 10),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    let up_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "up",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let down_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "down",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let toggle_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "toggle",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let all_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "all",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let none_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "none",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let push_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "push",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let cancel_key = keybindings::display_keys_joined(
        keybindings::ModeId::PushSelect,
        None,
        "cancel",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    lines.push(Line::from(Span::styled(
        format!(
            "{up_key}/{down_key}: nav | {toggle_key}: toggle | {all_key}/{none_key}: all/none | {push_key}: push | {cancel_key}: cancel"
        ),
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_conflicts_panel(frame: &mut Frame, state: &ConflictsState) {
    let area = frame.area();
    let list_height = state.files.len().clamp(1, 15);
    let popup_height = (5 + list_height) as u16;
    let popup_width = 60u16.min(area.width.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Conflicted Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(
        block.style(Style::default().bg(theme::POPUP_BG)),
        popup_area,
    );

    let mut lines: Vec<Line> = Vec::new();

    if state.files.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No conflicts found",
            Style::default().fg(Color::Green),
        )));
    } else {
        for (i, file) in state.files.iter().take(15).enumerate() {
            let marker = if i == state.selected_index {
                "> "
            } else {
                "  "
            };
            let style = if i == state.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(format!("{marker}{file}"), style)));
        }

        if state.files.len() > 15 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", state.files.len() - 15),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    let down_key = keybindings::display_keys_joined(
        keybindings::ModeId::Conflicts,
        None,
        "down",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let up_key = keybindings::display_keys_joined(
        keybindings::ModeId::Conflicts,
        None,
        "up",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let resolve_key = keybindings::display_keys_joined(
        keybindings::ModeId::Conflicts,
        None,
        "resolve",
        false,
        keybindings::KeyFormat::Space,
        "/",
    );
    let exit_keys = keybindings::display_keys_joined(
        keybindings::ModeId::Conflicts,
        None,
        "exit",
        true,
        keybindings::KeyFormat::Space,
        "/",
    );
    lines.push(Line::from(Span::styled(
        format!("{down_key}/{up_key}: navigate | {resolve_key}: resolve | {exit_keys}: exit"),
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_prefix_key_popup(frame: &mut Frame, mode: keybindings::ModeId, pending_key: char) {
    const KEY_SPACING: usize = 3; // "k  " (key char + 2 spaces)
    const TITLE_WRAPPER: usize = 4; // "g ()" around title
    const HORIZONTAL_PADDING: usize = 4;
    const FIXED_HEIGHT_LINES: usize = 4; // title + separator + 2 border lines
    const EDGE_MARGIN: u16 = 2;

    let Some(menu) = keybindings::prefix_menu(mode, pending_key) else {
        return;
    };

    let max_label_width = menu
        .items
        .iter()
        .map(|(_, label)| label.width())
        .max()
        .unwrap_or(0);
    let content_width = max_label_width + KEY_SPACING;
    let title_width = menu.title.width() + TITLE_WRAPPER;
    let popup_width = (content_width.max(title_width) + HORIZONTAL_PADDING) as u16;
    let popup_height = (menu.items.len() + FIXED_HEIGHT_LINES) as u16;

    let area = frame.area();
    let popup_area = Rect {
        x: area.width.saturating_sub(popup_width + EDGE_MARGIN),
        y: area.height.saturating_sub(popup_height + EDGE_MARGIN),
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(theme::PREFIX_POPUP_BG));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // title line
    lines.push(Line::from(Span::styled(
        format!("{} ({})", pending_key, menu.title),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));

    // separator
    lines.push(Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(Color::DarkGray),
    )));

    // key bindings
    for (key, label) in menu.items {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{key}  "),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(label, Style::default().fg(Color::White)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_toast(frame: &mut Frame, msg: &StatusMessage) {
    let color = match msg.kind {
        MessageKind::Info => Color::Blue,
        MessageKind::Success => Color::Green,
        MessageKind::Warning => Color::Yellow,
        MessageKind::Error => Color::Red,
    };

    let popup = Popup::new(msg.text.clone()).style(Style::default().fg(color).bg(theme::TOAST_BG));

    frame.render_widget(popup, frame.area());
}
