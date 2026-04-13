use super::super::{TreeNode, TreeTopology, VisibleEntry};
use ahash::{HashMap, HashSet};

pub(super) fn visible_scope<'a>(
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

pub(super) fn full_mode_entries(
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
                neighborhood: None,
            }
        })
        .collect()
}

pub(super) fn compact_mode_entries(
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
            neighborhood: None,
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
