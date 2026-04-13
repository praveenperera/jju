use super::super::super::{app::App, state::ModeState, tree::TreeNode};

pub(super) fn current_info(app: &App) -> String {
    if let ModeState::Rebasing(state) = &app.mode {
        let dest_name = destination_name(app, state.dest_cursor, true);
        let source_short: String = state.source_rev.chars().take(8).collect();
        return format!(" | {source_short}→{dest_name}");
    }

    if let ModeState::MovingBookmark(state) = &app.mode {
        let dest_name = destination_name(app, state.dest_cursor, false);
        let bookmark_name: String = state.bookmark_name.chars().take(12).collect();
        return format!(" | {bookmark_name}→{dest_name}");
    }

    if let ModeState::Squashing(state) = &app.mode {
        let dest_name = destination_name(app, state.dest_cursor, true);
        let source_short: String = state.source_rev.chars().take(8).collect();
        return format!(" | {source_short}→{dest_name}");
    }

    app.tree
        .current_node()
        .map(|node| format!(" | {}", primary_node_name(node, true)))
        .unwrap_or_default()
}

fn destination_name(app: &App, cursor: usize, allow_bookmarks: bool) -> String {
    app.tree
        .visible_entries()
        .get(cursor)
        .map(|entry| primary_node_name(&app.tree.nodes()[entry.node_index], allow_bookmarks))
        .unwrap_or_else(|| "?".to_string())
}

fn primary_node_name(node: &TreeNode, allow_bookmarks: bool) -> String {
    if allow_bookmarks && !node.bookmarks.is_empty() {
        node.bookmark_names().join(" ")
    } else {
        node.change_id.chars().take(8).collect()
    }
}
