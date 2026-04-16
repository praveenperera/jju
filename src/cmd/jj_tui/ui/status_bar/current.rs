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
        if state.source_revs.len() == 1 {
            let source_short: String = state.source_revs[0].chars().take(8).collect();
            return format!(" | {source_short}→{dest_name}");
        }

        return format!(" | {} revs→{dest_name}", state.source_revs.len());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::state::{ModeState, SquashState};
    use crate::cmd::jj_tui::test_support::{TestNodeKind, make_app_with_tree, make_tree};

    #[test]
    fn test_current_info_uses_single_squash_source_id() {
        let tree = make_tree(vec![
            TestNodeKind::Plain.make_node("aaaa1111", 0),
            TestNodeKind::Bookmarked(&["main"]).make_node("bbbb2222", 0),
        ]);
        let mut app = make_app_with_tree(tree);
        app.mode = ModeState::Squashing(SquashState {
            source_revs: vec!["aaaa1111".to_string()],
            dest_cursor: 1,
            op_before: String::new(),
        });

        assert_eq!(current_info(&app), " | aaaa1111→main");
    }

    #[test]
    fn test_current_info_uses_count_for_multi_squash() {
        let tree = make_tree(vec![
            TestNodeKind::Plain.make_node("aaaa1111", 0),
            TestNodeKind::Plain.make_node("bbbb2222", 0),
            TestNodeKind::Bookmarked(&["main"]).make_node("cccc3333", 0),
        ]);
        let mut app = make_app_with_tree(tree);
        app.mode = ModeState::Squashing(SquashState {
            source_revs: vec!["aaaa1111".to_string(), "bbbb2222".to_string()],
            dest_cursor: 2,
            op_before: String::new(),
        });

        assert_eq!(current_info(&app), " | 2 revs→main");
    }
}
