mod compact;
mod neighborhood;

use super::{TreeNode, TreeTopology, VisibleEntry};
use compact::{compact_mode_entries, full_mode_entries, visible_scope};
use neighborhood::neighborhood_entries;

pub(super) struct NeighborhoodFilter {
    pub anchor_index: usize,
    pub ancestor_limit: usize,
    pub descendant_limit: usize,
    pub sibling_depth_limit: usize,
}

pub(super) struct VisibleOptions {
    pub full_mode: bool,
    pub focused_root: Option<usize>,
    pub neighborhood: Option<NeighborhoodFilter>,
}

pub(super) fn compute_visible_entries(
    nodes: &[TreeNode],
    topology: &TreeTopology,
    options: VisibleOptions,
) -> Vec<VisibleEntry> {
    if let Some(neighborhood) = options.neighborhood {
        return neighborhood_entries(
            nodes.len(),
            topology,
            neighborhood.anchor_index,
            neighborhood.ancestor_limit,
            neighborhood.descendant_limit,
            neighborhood.sibling_depth_limit,
        );
    }

    let (filtered_nodes, base_depth) = visible_scope(nodes, topology, options.focused_root);

    if options.full_mode {
        full_mode_entries(&filtered_nodes, base_depth)
    } else {
        compact_mode_entries(&filtered_nodes, topology, options.full_mode)
    }
}

#[cfg(test)]
mod tests {
    use super::{NeighborhoodFilter, VisibleOptions, compute_visible_entries};
    use crate::cmd::jj_tui::test_support::TestNodeKind;
    use crate::cmd::jj_tui::tree::TreeTopology;

    fn visible_ids(
        entries: &[crate::cmd::jj_tui::tree::VisibleEntry],
        ids: &[&str],
    ) -> Vec<String> {
        entries
            .iter()
            .map(|entry| ids[entry.node_index].to_string())
            .collect()
    }

    #[test]
    fn neighborhood_shows_medium_mainline_window() {
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
                    descendant_limit: 2,
                    sibling_depth_limit: 2,
                }),
            },
        );

        assert_eq!(
            visible_ids(&entries, &ids),
            vec!["a", "b", "c", "d", "e", "f", "g"]
        );
    }

    #[test]
    fn neighborhood_includes_direct_forks_only() {
        let ids = vec!["a", "b", "c", "side1", "side2", "d", "e", "other-root"];
        let nodes = vec![
            TestNodeKind::Plain.make_node("a", 0),
            TestNodeKind::Plain.make_node("b", 1),
            TestNodeKind::Plain.make_node("c", 2),
            TestNodeKind::Plain.make_node("side1", 3),
            TestNodeKind::Plain.make_node("side2", 4),
            TestNodeKind::Plain.make_node("d", 3),
            TestNodeKind::Plain.make_node("e", 4),
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
                    descendant_limit: 2,
                    sibling_depth_limit: 2,
                }),
            },
        );

        assert_eq!(
            visible_ids(&entries, &ids),
            vec!["a", "b", "c", "side1", "side2", "d", "e"]
        );
        assert_eq!(entries[3].visual_depth, 3);
        assert_eq!(entries[4].visual_depth, 4);
    }

    #[test]
    fn neighborhood_stops_forward_path_at_fork() {
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
                    descendant_limit: 2,
                    sibling_depth_limit: 2,
                }),
            },
        );

        assert_eq!(
            visible_ids(&entries, &ids),
            vec!["a", "b", "c", "left", "right"]
        );
    }
}
