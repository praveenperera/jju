//! View model for tree rows - separates state computation from rendering

mod details;
mod operation;
mod row;

pub use self::details::RowDetails;
use self::operation::OperationViewBuilder;
pub use self::row::{Marker, TreeRowVm};
use super::app::App;
use super::state::ModeState;

/// Build the view model for all visible tree rows
pub fn build_tree_view(app: &App, _viewport_width: usize) -> Vec<TreeRowVm> {
    let builder = OperationViewBuilder::new(app);

    match &app.mode {
        ModeState::Rebasing(state) => builder.build_rebase_view(
            &state.source_rev,
            state.dest_cursor,
            state.rebase_type,
            state.allow_branches,
        ),
        ModeState::MovingBookmark(state) => {
            builder.build_bookmark_move_view(&state.bookmark_name, state.dest_cursor)
        }
        ModeState::Squashing(state) => {
            builder.build_squash_view(&state.source_rev, state.dest_cursor)
        }
        _ => builder.build_normal_view(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::preview::NodeRole;
    use crate::cmd::jj_tui::state::{ModeState, RebaseState, RebaseType};
    use crate::cmd::jj_tui::test_support::{TestNodeKind, make_app_with_tree, make_tree};
    use crate::jj_lib_helpers::CommitDetails;

    #[test]
    fn test_build_normal_view_cursor_tracking() {
        let tree = make_tree(vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
        ]);
        let mut app = make_app_with_tree(tree);
        app.tree.view.cursor = 1;

        let vms = build_tree_view(&app, 80);

        assert_eq!(vms.len(), 3);
        assert!(!vms[0].is_cursor);
        assert!(vms[1].is_cursor);
        assert!(!vms[2].is_cursor);
    }

    #[test]
    fn test_build_normal_view_selection_state() {
        let tree = make_tree(vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
        ]);
        let mut app = make_app_with_tree(tree);
        app.tree.view.selected.insert(0);
        app.tree.view.selected.insert(2);

        let vms = build_tree_view(&app, 80);

        assert!(vms[0].is_selected);
        assert!(!vms[1].is_selected);
        assert!(vms[2].is_selected);
    }

    #[test]
    fn test_build_rebase_view_roles() {
        let tree = make_tree(vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
        ]);
        let mut app = make_app_with_tree(tree);
        app.mode = ModeState::Rebasing(RebaseState {
            source_rev: "cccc".to_string(),
            dest_cursor: 0,
            rebase_type: RebaseType::Single,
            allow_branches: true,
        });

        let vms = build_tree_view(&app, 80);

        let source_vm = vms.iter().find(|vm| vm.change_id_prefix == "cccc").unwrap();
        let dest_vm = vms.iter().find(|vm| vm.change_id_prefix == "aaaa").unwrap();

        assert_eq!(source_vm.role, NodeRole::Source);
        assert_eq!(dest_vm.role, NodeRole::Destination);
        assert!(matches!(source_vm.marker, Some(Marker::Source)));
        assert!(matches!(dest_vm.marker, Some(Marker::Destination { .. })));
    }

    #[test]
    fn test_build_row_details_uses_loading_placeholder_while_pending() {
        let node = TestNodeKind::Plain.make_node("aaaa", 0);

        let details = details::build_row_details(&node, None);

        assert_eq!(details.author, "loading...");
        assert_eq!(details.timestamp, "loading...");
        assert_eq!(details.full_description, "loading...");
    }

    #[test]
    fn test_build_row_details_uses_hydrated_commit_metadata() {
        let mut node = TestNodeKind::Plain.make_node("aaaa", 0);
        node.commit_id = "1234567890abcdef".to_string();
        node.details = Some(CommitDetails {
            unique_commit_prefix_len: 8,
            full_description: "full body".to_string(),
            author_name: "Praveen".to_string(),
            author_email: "praveen@example.com".to_string(),
            timestamp: "2 days ago".to_string(),
        });

        let details = details::build_row_details(&node, None);

        assert_eq!(details.commit_id_prefix, "12345678");
        assert_eq!(details.commit_id_suffix, "90abcdef");
        assert_eq!(details.author, "Praveen <praveen@example.com>");
        assert_eq!(details.timestamp, "2 days ago");
        assert_eq!(details.full_description, "full body");
    }
}
