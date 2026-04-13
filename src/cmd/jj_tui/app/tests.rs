use super::{App, AppOptions, ModeState, apply_startup_options};
use crate::cmd::jj_tui::test_support::{TestNodeKind, make_app_with_tree, make_tree};
use crate::cmd::jj_tui::tree::TreeLoadScope;

fn visible_ids(app: &App) -> Vec<String> {
    app.tree
        .visible_entries()
        .iter()
        .map(|entry| app.tree.nodes()[entry.node_index].change_id.clone())
        .collect()
}

#[test]
fn startup_neighborhood_jumps_to_working_copy() {
    let mut nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("d", 3),
    ];
    nodes[2].is_working_copy = true;
    let mut tree = make_tree(nodes);

    apply_startup_options(
        &mut tree,
        AppOptions {
            start_in_neighborhood: true,
        },
    );

    assert!(tree.is_neighborhood_mode());
    assert_eq!(tree.view.load_scope, TreeLoadScope::Neighborhood);
    assert_eq!(
        tree.current_node().map(|node| node.change_id.as_str()),
        Some("c")
    );
}

#[test]
fn neighborhood_freezes_in_modal_modes_and_resumes_in_normal() {
    let nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("d", 3),
        TestNodeKind::Plain.make_node("e", 4),
        TestNodeKind::Plain.make_node("f", 5),
        TestNodeKind::Plain.make_node("g", 6),
        TestNodeKind::Plain.make_node("h", 7),
    ];
    let tree = make_tree(nodes);
    let mut app = make_app_with_tree(tree);

    app.tree.view.cursor = 4;
    app.tree.enable_neighborhood();
    let initial_visible = visible_ids(&app);

    app.mode = ModeState::Selecting;
    app.transition_neighborhood_mode(&ModeState::Normal);
    assert!(!app.tree.is_neighborhood_following_cursor());

    app.tree.move_cursor_up();
    assert_eq!(visible_ids(&app), initial_visible);
    assert_eq!(
        app.tree.current_node().map(|node| node.change_id.as_str()),
        Some("d")
    );

    app.mode = ModeState::Normal;
    app.transition_neighborhood_mode(&ModeState::Selecting);
    assert!(app.tree.is_neighborhood_following_cursor());
    assert_eq!(visible_ids(&app), vec!["a", "b", "c", "d", "e", "f"]);
    assert_eq!(
        app.tree.current_node().map(|node| node.change_id.as_str()),
        Some("d")
    );
}

#[test]
fn neighborhood_can_grow_and_shrink() {
    let nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("d", 3),
        TestNodeKind::Plain.make_node("e", 4),
        TestNodeKind::Plain.make_node("f", 5),
        TestNodeKind::Plain.make_node("g", 6),
        TestNodeKind::Plain.make_node("h", 7),
        TestNodeKind::Plain.make_node("i", 8),
    ];
    let tree = make_tree(nodes);
    let mut app = make_app_with_tree(tree);

    app.tree.view.cursor = 4;
    app.tree.enable_neighborhood();
    assert_eq!(visible_ids(&app), vec!["a", "b", "c", "d", "e", "f", "g"]);

    assert!(app.tree.expand_neighborhood());
    assert_eq!(
        visible_ids(&app),
        vec!["a", "b", "c", "d", "e", "f", "g", "h", "i"]
    );

    assert!(app.tree.shrink_neighborhood());
    assert_eq!(visible_ids(&app), vec!["a", "b", "c", "d", "e", "f", "g"]);
}
