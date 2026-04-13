use super::TreeRefreshRemapper;
use crate::cmd::jj_tui::test_support::{TestNodeKind, make_tree};
use crate::cmd::jj_tui::tree::{NeighborhoodState, ViewMode};

#[test]
fn restore_focus_stack_reapplies_focus_to_matching_change_ids() {
    let mut old_tree = make_tree(vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
    ]);
    old_tree.view.cursor = 1;
    old_tree.focus_on(1);
    let remapper = TreeRefreshRemapper::capture(&old_tree);

    let mut refreshed_tree = make_tree(vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("d", 2),
    ]);
    remapper.restore(&mut refreshed_tree);

    assert!(refreshed_tree.is_focused());
    assert_eq!(
        refreshed_tree
            .current_node()
            .map(|node| node.change_id.as_str()),
        Some("b")
    );
}

#[test]
fn restore_neighborhood_keeps_anchor_and_history() {
    let mut old_tree = make_tree(vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
    ]);
    old_tree.view.cursor = 1;
    old_tree.set_view_mode(ViewMode::Neighborhood(NeighborhoodState {
        anchor_change_id: "b".to_string(),
        history: vec!["a".to_string()],
        level: 2,
    }));
    let remapper = TreeRefreshRemapper::capture(&old_tree);

    let mut refreshed_tree = make_tree(vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
    ]);
    remapper.restore(&mut refreshed_tree);

    assert_eq!(
        refreshed_tree.neighborhood_state().map(|state| (
            state.anchor_change_id.clone(),
            state.history.clone(),
            state.level
        )),
        Some(("b".to_string(), vec!["a".to_string()], 2))
    );
    assert_eq!(
        refreshed_tree
            .current_node()
            .map(|node| node.change_id.as_str()),
        Some("b")
    );
}

#[test]
fn restore_cursor_falls_back_to_parent_change() {
    let parent = TestNodeKind::Plain.make_node("parent", 0);
    let mut child = TestNodeKind::Plain.make_node("child", 1);
    child.parent_ids = vec!["parent".to_string()];

    let mut old_tree = make_tree(vec![parent.clone(), child]);
    old_tree.view.cursor = 1;
    let remapper = TreeRefreshRemapper::capture(&old_tree);

    let mut refreshed_tree = make_tree(vec![parent]);
    remapper.restore(&mut refreshed_tree);

    assert_eq!(
        refreshed_tree
            .current_node()
            .map(|node| node.change_id.as_str()),
        Some("parent")
    );
}
