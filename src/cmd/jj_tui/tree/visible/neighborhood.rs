use super::super::{TreeTopology, VisibleEntry};
use ahash::{HashMap, HashSet};

pub(super) fn neighborhood_entries(
    node_count: usize,
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
    descendant_limit: usize,
    sibling_depth_limit: usize,
) -> Vec<VisibleEntry> {
    if anchor_index >= node_count {
        return Vec::new();
    }

    let visible_nodes = neighborhood_visible_nodes(
        node_count,
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
