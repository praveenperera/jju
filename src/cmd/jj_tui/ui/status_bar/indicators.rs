use super::super::super::{
    app::App,
    keybindings,
    state::{ModeState, RebaseType},
};

pub(super) fn mode_indicator(app: &App) -> &'static str {
    match &app.mode {
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
    }
}

pub(super) fn full_indicator(app: &App) -> &'static str {
    if app.tree.view.full_mode {
        " [FULL]"
    } else {
        ""
    }
}

pub(super) fn neighborhood_indicator(app: &App) -> String {
    app.tree
        .neighborhood_state()
        .map(|state| match state.local_level() {
            Some(level) => format!(" [NEIGHBORHOOD:{}]", level + 1),
            None => " [NEIGHBORHOOD:FULL]".to_string(),
        })
        .unwrap_or_default()
}

pub(super) fn split_indicator(app: &App) -> &'static str {
    if app.split_view { " [SPLIT]" } else { "" }
}

pub(super) fn focus_indicator(app: &App) -> String {
    if !app.tree.is_focused() {
        return String::new();
    }

    let depth = app.tree.focus_depth();
    let focused_name = app
        .tree
        .focused_node()
        .map(display_node_name)
        .unwrap_or_default();
    format!(" [ZOOM:{depth}→{focused_name}]")
}

pub(super) fn pending_indicator(app: &App) -> String {
    match app.pending_key {
        Some(prefix) if keybindings::is_known_prefix(prefix) => format!(" {prefix}-"),
        _ => String::new(),
    }
}

pub(super) fn selection_indicator(app: &App) -> String {
    if app.tree.view.selected.is_empty() {
        String::new()
    } else {
        format!(" [{}sel]", app.tree.view.selected.len())
    }
}

pub(super) fn hints(app: &App) -> String {
    let rebase_allow_branches = match &app.mode {
        ModeState::Rebasing(state) => Some(state.allow_branches),
        _ => None,
    };

    keybindings::status_bar_hints(&keybindings::StatusHintContext {
        mode: keybindings::mode_id_from_state(&app.mode),
        has_selection: !app.tree.view.selected.is_empty(),
        has_focus: app.tree.is_focused(),
        neighborhood_active: app.tree.is_neighborhood_mode(),
        current_has_bookmark: app.current_has_bookmark(),
        rebase_allow_branches,
    })
}

fn display_node_name(node: &crate::cmd::jj_tui::tree::TreeNode) -> String {
    if !node.bookmarks.is_empty() {
        node.bookmark_names().first().cloned().unwrap_or_default()
    } else {
        node.change_id.chars().take(8).collect()
    }
}
