use super::startup::apply_startup_options;
use super::{App, AppOptions};
use crate::cmd::jj_tui::test_support::{TestNodeKind, make_app_with_tree, make_tree};
use crate::cmd::jj_tui::tree::{
    NeighborhoodExtent, NeighborhoodResize, NeighborhoodState, TreeLoadScope, ViewMode,
};

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
fn neighborhood_stays_fixed_while_cursor_moves() {
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
    let initial_visible = visible_ids(&app);

    app.tree.move_cursor_up();
    app.tree.move_cursor_down();
    app.tree.page_down(2);

    assert_eq!(visible_ids(&app), initial_visible);
}

#[test]
fn neighborhood_can_grow_and_shrink_previews() {
    let ids = vec!["a", "b", "c", "left1", "left2", "left3", "main1", "main2"];
    let nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("left1", 3),
        TestNodeKind::Plain.make_node("left2", 4),
        TestNodeKind::Plain.make_node("left3", 5),
        TestNodeKind::Plain.make_node("main1", 3),
        TestNodeKind::Plain.make_node("main2", 4),
    ];
    let tree = make_tree(nodes);
    let mut app = make_app_with_tree(tree);

    app.tree.view.cursor = 2;
    app.tree.enable_neighborhood();

    let visible_before = visible_ids(&app);
    assert_eq!(
        visible_before,
        vec!["a", "b", "c", "left1", "left2", "main1", "main2"]
    );

    assert_eq!(
        app.tree.expand_neighborhood(),
        NeighborhoodResize::Reprojected
    );
    assert_eq!(
        visible_ids(&app),
        vec!["a", "b", "c", "left1", "left2", "left3", "main1", "main2"]
    );

    let left_preview = app
        .tree
        .visible_entries()
        .iter()
        .find(|entry| ids[entry.node_index] == "left3")
        .and_then(|entry| entry.neighborhood.as_ref())
        .map(|entry| entry.hidden_count);
    assert_eq!(left_preview, Some(0));

    assert_eq!(
        app.tree.shrink_neighborhood(),
        NeighborhoodResize::Reprojected
    );
    assert_eq!(visible_ids(&app), visible_before);
}

#[test]
fn neighborhood_expand_reaches_full_tree_extent() {
    let nodes = (0..40)
        .map(|depth| TestNodeKind::Plain.make_node(&format!("n{depth:02}"), depth))
        .collect();
    let tree = make_tree(nodes);
    let mut app = make_app_with_tree(tree);

    app.tree.view.cursor = 35;
    app.tree.enable_neighborhood();

    for _ in 0..6 {
        assert_eq!(
            app.tree.expand_neighborhood(),
            NeighborhoodResize::Reprojected
        );
    }

    assert_eq!(
        app.tree.expand_neighborhood(),
        NeighborhoodResize::ScopeChanged
    );
    assert_eq!(app.tree.view.load_scope, TreeLoadScope::Stack);
    assert_eq!(
        app.tree
            .neighborhood_state()
            .map(|state| state.extent.clone()),
        Some(NeighborhoodExtent::FullTree)
    );

    assert_eq!(
        app.tree.shrink_neighborhood(),
        NeighborhoodResize::ScopeChanged
    );
    assert_eq!(app.tree.view.load_scope, TreeLoadScope::Neighborhood);
    assert_eq!(
        app.tree
            .neighborhood_state()
            .map(|state| state.extent.clone()),
        Some(NeighborhoodExtent::Local(6))
    );
}

#[test]
fn full_tree_neighborhood_matches_tree_projection() {
    let nodes: Vec<_> = (0..40)
        .map(|depth| TestNodeKind::Plain.make_node(&format!("n{depth:02}"), depth))
        .collect();
    let mut tree = make_tree(nodes.clone());
    tree.view.cursor = 35;

    let mut tree_mode = make_tree(nodes);
    tree_mode.view.cursor = 35;
    let tree_visible = visible_ids(&make_app_with_tree(tree_mode));
    tree.set_view_mode(ViewMode::Neighborhood(NeighborhoodState {
        anchor_change_id: "n35".to_string(),
        history: Vec::new(),
        extent: NeighborhoodExtent::FullTree,
    }));

    let app = make_app_with_tree(tree);
    assert_eq!(visible_ids(&app), tree_visible);
    assert_eq!(app.tree.view.load_scope, TreeLoadScope::Stack);
}

#[test]
fn neighborhood_can_enter_and_exit_preview_paths() {
    let nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("left1", 3),
        TestNodeKind::Plain.make_node("left2", 4),
        TestNodeKind::Plain.make_node("main1", 3),
        TestNodeKind::Plain.make_node("main2", 4),
    ];
    let tree = make_tree(nodes);
    let mut app = make_app_with_tree(tree);

    app.tree.view.cursor = 2;
    app.tree.enable_neighborhood();
    app.tree.view.cursor = 3;

    assert!(app.tree.current_entry_is_neighborhood_preview());
    assert!(app.tree.enter_neighborhood_path());
    assert_eq!(
        app.tree
            .neighborhood_state()
            .map(|state| (state.anchor_change_id.as_str(), state.history.clone())),
        Some(("left1", vec!["c".to_string()]))
    );
    assert_eq!(
        visible_ids(&app),
        vec!["a", "b", "c", "left1", "left2", "main1", "main2"]
    );

    assert!(app.tree.exit_neighborhood_path());
    assert_eq!(
        app.tree
            .neighborhood_state()
            .map(|state| state.anchor_change_id.as_str()),
        Some("c")
    );
    assert_eq!(
        visible_ids(&app),
        vec!["a", "b", "c", "left1", "left2", "main1", "main2"]
    );
}
