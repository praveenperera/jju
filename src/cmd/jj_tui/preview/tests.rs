use super::test_support::{find_slot, make_tree, visible_topology};
use super::*;
use crate::cmd::jj_tui::test_support::TestNodeKind;
use crate::cmd::jj_tui::tree::{
    TreeLoadScope, TreeProjection, TreeSnapshot, TreeState, TreeTopology, TreeViewState, ViewMode,
    VisibleEntry,
};

#[test]
fn test_from_tree_linear() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
            TestNodeKind::Plain.make_node("dddd", 3),
        ],
        true,
    );

    let relations = visible_topology(&tree);
    assert_eq!(relations.parent_of(0), None);
    assert_eq!(relations.parent_of(1), Some(0));
    assert_eq!(relations.parent_of(2), Some(1));
    assert_eq!(relations.parent_of(3), Some(2));
    assert_eq!(relations.children_of(0), &[1]);
    assert_eq!(relations.children_of(1), &[2]);
    assert_eq!(relations.children_of(2), &[3]);
    assert_eq!(relations.children_of(3), &[] as &[usize]);
}

#[test]
fn test_from_tree_forked() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
            TestNodeKind::Plain.make_node("dddd", 1),
        ],
        true,
    );

    let relations = visible_topology(&tree);
    assert_eq!(relations.parent_of(0), None);
    assert_eq!(relations.parent_of(1), Some(0));
    assert_eq!(relations.parent_of(2), Some(1));
    assert_eq!(relations.parent_of(3), Some(0));

    let children_of_a = relations.children_of(0);
    assert!(children_of_a.contains(&1));
    assert!(children_of_a.contains(&3));
    assert_eq!(children_of_a.len(), 2);
}

#[test]
fn test_descendants() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
        ],
        true,
    );

    let relations = visible_topology(&tree);
    let desc_a = relations.descendants(0);
    assert!(desc_a.contains(&1));
    assert!(desc_a.contains(&2));
    assert_eq!(desc_a.len(), 2);

    let desc_b = relations.descendants(1);
    assert!(desc_b.contains(&2));
    assert_eq!(desc_b.len(), 1);

    let desc_c = relations.descendants(2);
    assert!(desc_c.is_empty());
}

#[test]
fn test_rebase_with_descendants() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
            TestNodeKind::Plain.make_node("dddd", 2),
        ],
        true,
    );

    let preview = PreviewBuilder::new(&tree).rebase_preview(
        NodeId(1),
        NodeId(0),
        PreviewRebaseType::WithDescendants,
        true,
    );

    assert_eq!(preview.slots.len(), 4);
    let slot_a = find_slot(&preview.slots, 0);
    let slot_b = find_slot(&preview.slots, 1);
    let slot_c = find_slot(&preview.slots, 2);
    let slot_d = find_slot(&preview.slots, 3);

    assert_eq!(slot_a.visual_depth, 0);
    assert_eq!(slot_b.visual_depth, 1);
    assert_eq!(slot_c.visual_depth, 2);
    assert_eq!(slot_d.visual_depth, 2);
    assert_eq!(slot_b.role, NodeRole::Source);
    assert_eq!(slot_c.role, NodeRole::Moving);
    assert_eq!(slot_d.role, NodeRole::Moving);
}

#[test]
fn test_non_full_mode_with_hidden_nodes() {
    let nodes = vec![
        TestNodeKind::Plain.make_node("aaaa", 0),
        TestNodeKind::Plain.make_node("cccc", 2),
        TestNodeKind::Plain.make_node("dddd", 3),
    ];
    let visible_entries = vec![
        VisibleEntry {
            node_index: 0,
            visual_depth: 0,
            has_separator_before: false,
            neighborhood: None,
        },
        VisibleEntry {
            node_index: 1,
            visual_depth: 1,
            has_separator_before: false,
            neighborhood: None,
        },
        VisibleEntry {
            node_index: 2,
            visual_depth: 2,
            has_separator_before: false,
            neighborhood: None,
        },
    ];
    let topology = TreeTopology::from_nodes(&nodes);
    let snapshot = TreeSnapshot { nodes, topology };
    let view = TreeViewState {
        full_mode: false,
        view_mode: ViewMode::Tree,
        ..TreeViewState::new(TreeLoadScope::Stack)
    };
    let projection = TreeProjection { visible_entries };
    let tree = TreeState {
        snapshot,
        view,
        projection,
    };

    let relations = visible_topology(&tree);
    assert_eq!(relations.parent_of(0), None);
    assert_eq!(relations.parent_of(1), Some(0));
    assert_eq!(relations.parent_of(2), Some(1));
}

#[test]
fn test_rebase_subtree_to_different_parent() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
            TestNodeKind::Plain.make_node("dddd", 3),
        ],
        true,
    );

    let preview = PreviewBuilder::new(&tree).rebase_preview(
        NodeId(2),
        NodeId(0),
        PreviewRebaseType::WithDescendants,
        true,
    );

    let slot_a = find_slot(&preview.slots, 0);
    let slot_b = find_slot(&preview.slots, 1);
    let slot_c = find_slot(&preview.slots, 2);
    let slot_d = find_slot(&preview.slots, 3);

    assert_eq!(slot_a.visual_depth, 0);
    assert_eq!(slot_a.role, NodeRole::Destination);
    assert_eq!(slot_b.visual_depth, 1);
    assert_eq!(slot_b.role, NodeRole::Normal);
    assert_eq!(slot_c.visual_depth, 1);
    assert_eq!(slot_c.role, NodeRole::Source);
    assert_eq!(slot_d.visual_depth, 2);
    assert_eq!(slot_d.role, NodeRole::Moving);
}

#[test]
fn test_rebase_inline_mode_linear_chain() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
            TestNodeKind::Plain.make_node("dddd", 3),
        ],
        true,
    );

    let preview = PreviewBuilder::new(&tree).rebase_preview(
        NodeId(2),
        NodeId(0),
        PreviewRebaseType::WithDescendants,
        false,
    );

    let slot_a = find_slot(&preview.slots, 0);
    let slot_b = find_slot(&preview.slots, 1);
    let slot_c = find_slot(&preview.slots, 2);
    let slot_d = find_slot(&preview.slots, 3);

    assert_eq!(slot_a.visual_depth, 0);
    assert_eq!(slot_c.visual_depth, 1);
    assert_eq!(slot_d.visual_depth, 2);
    assert_eq!(slot_b.visual_depth, 3);
    assert_eq!(slot_a.role, NodeRole::Destination);
    assert_eq!(slot_c.role, NodeRole::Source);
    assert_eq!(slot_d.role, NodeRole::Moving);
    assert_eq!(slot_b.role, NodeRole::Normal);
}

#[test]
fn test_rebase_fork_mode() {
    let tree = make_tree(
        vec![
            TestNodeKind::Plain.make_node("aaaa", 0),
            TestNodeKind::Plain.make_node("bbbb", 1),
            TestNodeKind::Plain.make_node("cccc", 2),
            TestNodeKind::Plain.make_node("dddd", 3),
        ],
        true,
    );

    let preview = PreviewBuilder::new(&tree).rebase_preview(
        NodeId(2),
        NodeId(0),
        PreviewRebaseType::WithDescendants,
        true,
    );

    let slot_b = find_slot(&preview.slots, 1);
    let slot_c = find_slot(&preview.slots, 2);
    let slot_d = find_slot(&preview.slots, 3);

    assert_eq!(slot_b.visual_depth, 1);
    assert_eq!(slot_c.visual_depth, 1);
    assert_eq!(slot_d.visual_depth, 2);
}
