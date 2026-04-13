use super::{NeighborhoodFilter, VisibleOptions, compute_visible_entries};
use crate::cmd::jj_tui::test_support::TestNodeKind;
use crate::cmd::jj_tui::tree::{TreeTopology, VisibleEntry};

fn visible_ids(entries: &[VisibleEntry], ids: &[&str]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| ids[entry.node_index].to_string())
        .collect()
}

#[test]
fn neighborhood_shows_all_revs_down_on_linear_path() {
    let mut nodes = Vec::new();
    let mut ids = Vec::new();

    for (depth, id) in ["a", "b", "c", "d", "e", "f", "g", "h"]
        .into_iter()
        .enumerate()
    {
        ids.push(id);
        nodes.push(TestNodeKind::Plain.make_node(id, depth));
    }

    let topology = TreeTopology::from_nodes(&nodes);
    let entries = compute_visible_entries(
        &nodes,
        &topology,
        VisibleOptions {
            full_mode: true,
            focused_root: None,
            neighborhood: Some(NeighborhoodFilter {
                anchor_index: 4,
                ancestor_limit: 4,
                preview_depth_limit: 2,
            }),
        },
    );

    assert_eq!(
        visible_ids(&entries, &ids),
        vec!["a", "b", "c", "d", "e", "f", "g", "h"]
    );
}

#[test]
fn neighborhood_previews_sibling_paths_with_hidden_counts() {
    let ids = vec![
        "a",
        "b",
        "c",
        "side1",
        "side2",
        "side3",
        "main1",
        "main2",
        "other-root",
    ];
    let nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("side1", 3),
        TestNodeKind::Plain.make_node("side2", 4),
        TestNodeKind::Plain.make_node("side3", 5),
        TestNodeKind::Plain.make_node("main1", 3),
        TestNodeKind::Plain.make_node("main2", 4),
        TestNodeKind::Plain.make_node("other-root", 0),
    ];

    let topology = TreeTopology::from_nodes(&nodes);
    let entries = compute_visible_entries(
        &nodes,
        &topology,
        VisibleOptions {
            full_mode: true,
            focused_root: None,
            neighborhood: Some(NeighborhoodFilter {
                anchor_index: 2,
                ancestor_limit: 4,
                preview_depth_limit: 2,
            }),
        },
    );

    assert_eq!(
        visible_ids(&entries, &ids),
        vec!["a", "b", "c", "side1", "side2", "main1", "main2"]
    );
    assert_eq!(entries[3].visual_depth, 3);
    assert_eq!(entries[4].visual_depth, 4);

    let preview = entries[4].neighborhood.as_ref().unwrap();
    assert!(preview.is_preview);
    assert_eq!(preview.hidden_count, 1);
}

#[test]
fn neighborhood_stops_mainline_at_first_fork_and_previews_all_children() {
    let ids = vec!["a", "b", "c", "left", "right"];
    let nodes = vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 1),
        TestNodeKind::Plain.make_node("c", 2),
        TestNodeKind::Plain.make_node("left", 3),
        TestNodeKind::Plain.make_node("right", 3),
    ];

    let topology = TreeTopology::from_nodes(&nodes);
    let entries = compute_visible_entries(
        &nodes,
        &topology,
        VisibleOptions {
            full_mode: true,
            focused_root: None,
            neighborhood: Some(NeighborhoodFilter {
                anchor_index: 2,
                ancestor_limit: 4,
                preview_depth_limit: 2,
            }),
        },
    );

    assert_eq!(
        visible_ids(&entries, &ids),
        vec!["a", "b", "c", "left", "right"]
    );
    assert!(entries[3].neighborhood.as_ref().unwrap().is_preview);
    assert!(entries[4].neighborhood.as_ref().unwrap().is_preview);
}
