use super::super::app::App;
use super::super::keybindings;
use super::super::state::{ModeState, RebaseType};
use super::super::theme;
use super::{Frame, Rect};
use ratatui::{
    style::{Color, Style},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

pub(super) fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mode_indicator = match &app.mode {
        ModeState::Normal => "NORMAL",
        ModeState::Help(..) => "HELP",
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
        ModeState::BookmarkSelect(_) => "SELECT BM",
        ModeState::BookmarkPicker(_) => "PICK BM",
        ModeState::PushSelect(_) => "PUSH SELECT",
        ModeState::Squashing(_) => "SQUASH",
        ModeState::Conflicts(_) => "CONFLICTS",
    };

    let full_indicator = if app.tree.view.full_mode {
        " [FULL]"
    } else {
        ""
    };
    let neighborhood_indicator = app
        .tree
        .neighborhood_state()
        .map(|state| format!(" [NEIGHBORHOOD:{}]", state.level + 1))
        .unwrap_or_default();
    let split_indicator = if app.split_view { " [SPLIT]" } else { "" };

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

    let pending_indicator = match app.pending_key {
        Some(p) if keybindings::is_known_prefix(p) => format!(" {p}-"),
        _ => String::new(),
    };

    let selection_indicator = if !app.tree.view.selected.is_empty() {
        format!(" [{}sel]", app.tree.view.selected.len())
    } else {
        String::new()
    };

    let current_info = current_info(app);

    let mode_id = keybindings::mode_id_from_state(&app.mode);
    let rebase_allow_branches = match &app.mode {
        ModeState::Rebasing(state) => Some(state.allow_branches),
        _ => None,
    };
    let hints = keybindings::status_bar_hints(&keybindings::StatusHintContext {
        mode: mode_id,
        has_selection: !app.tree.view.selected.is_empty(),
        has_focus: app.tree.is_focused(),
        neighborhood_active: app.tree.is_neighborhood_mode(),
        current_has_bookmark: app.current_has_bookmark(),
        rebase_allow_branches,
    });

    let left = format!(
        " {mode_indicator}{full_indicator}{neighborhood_indicator}{split_indicator}{focus_indicator}{pending_indicator}{selection_indicator}{current_info}"
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

fn current_info(app: &App) -> String {
    if let ModeState::Rebasing(state) = &app.mode {
        let dest_name = app
            .tree
            .visible_entries()
            .get(state.dest_cursor)
            .map(|entry| {
                let node = &app.tree.nodes()[entry.node_index];
                if node.bookmarks.is_empty() {
                    node.change_id.chars().take(8).collect::<String>()
                } else {
                    node.bookmark_names().join(" ")
                }
            })
            .unwrap_or_else(|| "?".to_string());
        let source_short: String = state.source_rev.chars().take(8).collect();
        format!(" | {source_short}→{dest_name}")
    } else if let ModeState::MovingBookmark(state) = &app.mode {
        let dest_name = app
            .tree
            .visible_entries()
            .get(state.dest_cursor)
            .map(|entry| {
                let node = &app.tree.nodes()[entry.node_index];
                node.change_id.chars().take(8).collect::<String>()
            })
            .unwrap_or_else(|| "?".to_string());
        let bookmark_name: String = state.bookmark_name.chars().take(12).collect();
        format!(" | {bookmark_name}→{dest_name}")
    } else if let ModeState::Squashing(state) = &app.mode {
        let dest_name = app
            .tree
            .visible_entries()
            .get(state.dest_cursor)
            .map(|entry| {
                let node = &app.tree.nodes()[entry.node_index];
                if node.bookmarks.is_empty() {
                    node.change_id.chars().take(8).collect::<String>()
                } else {
                    node.bookmark_names().join(" ")
                }
            })
            .unwrap_or_else(|| "?".to_string());
        let source_short: String = state.source_rev.chars().take(8).collect();
        format!(" | {source_short}→{dest_name}")
    } else {
        app.tree
            .current_node()
            .map(|node| {
                let name = if node.bookmarks.is_empty() {
                    node.change_id.clone()
                } else {
                    node.bookmark_names().join(" ")
                };
                format!(" | {name}")
            })
            .unwrap_or_default()
    }
}
