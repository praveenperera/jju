use super::{TreeNode, TreeTopology, VisibleEntry};
use ahash::{HashMap, HashSet};

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
            nodes,
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

fn visible_scope<'a>(
    nodes: &'a [TreeNode],
    topology: &TreeTopology,
    focused_root: Option<usize>,
) -> (Vec<(usize, &'a TreeNode)>, usize) {
    let Some(root_index) = focused_root else {
        return (nodes.iter().enumerate().collect(), 0);
    };
    if root_index >= nodes.len() {
        return (Vec::new(), 0);
    }

    let root_depth = nodes[root_index].depth;
    let scoped_nodes = topology
        .subtree_nodes_in_order(root_index)
        .into_iter()
        .map(|node_index| (node_index, &nodes[node_index]))
        .collect();
    (scoped_nodes, root_depth)
}

fn full_mode_entries(
    filtered_nodes: &[(usize, &TreeNode)],
    base_depth: usize,
) -> Vec<VisibleEntry> {
    let mut seen_root = false;
    filtered_nodes
        .iter()
        .map(|(node_index, node)| {
            let visual_depth = node.depth.saturating_sub(base_depth);
            let has_separator_before = visual_depth == 0 && seen_root;
            if visual_depth == 0 {
                seen_root = true;
            }
            VisibleEntry {
                node_index: *node_index,
                visual_depth,
                has_separator_before,
            }
        })
        .collect()
}

fn compact_mode_entries(
    filtered_nodes: &[(usize, &TreeNode)],
    topology: &TreeTopology,
    full_mode: bool,
) -> Vec<VisibleEntry> {
    let visible_nodes: Vec<usize> = filtered_nodes
        .iter()
        .filter_map(|(node_index, node)| node.is_visible(full_mode).then_some(*node_index))
        .collect();
    let visible_set: HashSet<usize> = visible_nodes.iter().copied().collect();
    let mut visual_depths: HashMap<usize, usize> = HashMap::default();
    let mut entries = Vec::new();
    let mut seen_root = false;

    for (node_index, node) in filtered_nodes {
        if !node.is_visible(full_mode) {
            continue;
        }

        let visual_depth =
            visible_parent_depth(*node_index, topology, &visible_set, &visual_depths);
        visual_depths.insert(*node_index, visual_depth);

        let has_separator_before = visual_depth == 0 && seen_root;
        if visual_depth == 0 {
            seen_root = true;
        }

        entries.push(VisibleEntry {
            node_index: *node_index,
            visual_depth,
            has_separator_before,
        });
    }

    entries
}

fn neighborhood_entries(
    nodes: &[TreeNode],
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
    descendant_limit: usize,
    sibling_depth_limit: usize,
) -> Vec<VisibleEntry> {
    if anchor_index >= nodes.len() {
        return Vec::new();
    }

    let visible_nodes = neighborhood_visible_nodes(
        nodes.len(),
        topology,
        anchor_index,
        ancestor_limit,
        descendant_limit,
        sibling_depth_limit,
    );
    projected_entries(topology, &visible_nodes)
}

fn neighborhood_visible_nodes(
    node_count: usize,
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
    descendant_limit: usize,
    sibling_depth_limit: usize,
) -> Vec<usize> {
    let mainline = mainline_window(topology, anchor_index, ancestor_limit, descendant_limit);
    let mainline_set: HashSet<usize> = mainline.iter().copied().collect();
    let mainline_edges: HashMap<usize, usize> = mainline
        .windows(2)
        .map(|window| (window[0], window[1]))
        .collect();
    let mut visible_set = mainline_set.clone();

    for &node_index in &mainline {
        let mainline_child = mainline_edges.get(&node_index).copied();
        let child_indices = topology.children_of(node_index);
        if mainline_child.is_none() && child_indices.len() <= 1 {
            continue;
        }

        for &child_index in child_indices {
            if Some(child_index) == mainline_child {
                continue;
            }
            add_branch_nodes(
                topology,
                child_index,
                sibling_depth_limit,
                &mut visible_set,
                &mainline_set,
            );
        }
    }

    (0..node_count)
        .filter(|node_index| visible_set.contains(node_index))
        .collect()
}

fn mainline_window(
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
    descendant_limit: usize,
) -> Vec<usize> {
    let mut ancestors = Vec::new();
    let mut current = anchor_index;
    for _ in 0..ancestor_limit {
        let Some(parent_index) = topology.parent_of(current) else {
            break;
        };
        ancestors.push(parent_index);
        current = parent_index;
    }
    ancestors.reverse();

    let mut mainline = ancestors;
    mainline.push(anchor_index);

    let mut current = anchor_index;
    for _ in 0..descendant_limit {
        let [child_index] = topology.children_of(current) else {
            break;
        };
        mainline.push(*child_index);
        current = *child_index;
    }

    mainline
}

fn add_branch_nodes(
    topology: &TreeTopology,
    branch_root: usize,
    depth_limit: usize,
    visible_set: &mut HashSet<usize>,
    mainline_set: &HashSet<usize>,
) {
    let mut current = branch_root;
    let mut remaining = depth_limit;

    while remaining > 0 {
        if !visible_set.insert(current) {
            break;
        }
        remaining -= 1;

        let [next_child] = topology.children_of(current) else {
            break;
        };
        if mainline_set.contains(next_child) {
            break;
        }

        current = *next_child;
    }
}

fn projected_entries(topology: &TreeTopology, visible_nodes: &[usize]) -> Vec<VisibleEntry> {
    let projected = topology.project_visible(visible_nodes);
    let mut visual_depths: HashMap<usize, usize> = HashMap::default();
    let mut entries = Vec::with_capacity(visible_nodes.len());
    let mut seen_root = false;

    for &node_index in visible_nodes {
        let visual_depth = projected
            .parent_of(node_index)
            .and_then(|parent_index| visual_depths.get(&parent_index).copied())
            .map_or(0, |depth| depth + 1);
        visual_depths.insert(node_index, visual_depth);

        let has_separator_before = visual_depth == 0 && seen_root;
        if visual_depth == 0 {
            seen_root = true;
        }

        entries.push(VisibleEntry {
            node_index,
            visual_depth,
            has_separator_before,
        });
    }

    entries
}

fn visible_parent_depth(
    node_index: usize,
    topology: &TreeTopology,
    visible_set: &HashSet<usize>,
    visual_depths: &HashMap<usize, usize>,
) -> usize {
    let mut current_parent = topology.parent_of(node_index);
    while let Some(parent_index) = current_parent {
        if visible_set.contains(&parent_index) {
            return visual_depths
                .get(&parent_index)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
        }
        current_parent = topology.parent_of(parent_index);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::{NeighborhoodFilter, VisibleOptions, compute_visible_entries};
    use crate::cmd::jj_tui::test_support::make_node;
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
            nodes.push(make_node(id, depth));
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
            make_node("a", 0),
            make_node("b", 1),
            make_node("c", 2),
            make_node("side1", 3),
            make_node("side2", 4),
            make_node("d", 3),
            make_node("e", 4),
            make_node("other-root", 0),
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
            make_node("a", 0),
            make_node("b", 1),
            make_node("c", 2),
            make_node("left", 3),
            make_node("right", 3),
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
