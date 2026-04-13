mod mainline;
mod preview;
mod project;

use super::super::{TreeTopology, VisibleEntry};
use ahash::{HashMap, HashSet};

pub(super) fn neighborhood_entries(
    node_count: usize,
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
    preview_depth_limit: usize,
) -> Vec<VisibleEntry> {
    if anchor_index >= node_count {
        return Vec::new();
    }

    project::projected_entries(
        topology,
        neighborhood_projection(
            node_count,
            topology,
            anchor_index,
            ancestor_limit,
            preview_depth_limit,
        ),
    )
}

struct NeighborhoodProjection {
    visible_nodes: Vec<usize>,
    preview_nodes: HashSet<usize>,
    preview_hidden_counts: HashMap<usize, usize>,
}

fn neighborhood_projection(
    node_count: usize,
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
    preview_depth_limit: usize,
) -> NeighborhoodProjection {
    let mainline = mainline::mainline_path(topology, anchor_index, ancestor_limit);
    let mainline_set: HashSet<usize> = mainline.iter().copied().collect();
    let mainline_edges: HashMap<usize, usize> = mainline
        .windows(2)
        .map(|window| (window[0], window[1]))
        .collect();
    let mut visible_set = mainline_set.clone();
    let mut preview_nodes = HashSet::default();
    let mut preview_hidden_counts = HashMap::default();

    for &node_index in &mainline {
        let active_child = mainline_edges.get(&node_index).copied();

        for &child_index in topology.children_of(node_index) {
            if Some(child_index) == active_child {
                continue;
            }

            let preview = preview::preview_branch(topology, child_index, preview_depth_limit);
            for &preview_node in &preview.nodes {
                visible_set.insert(preview_node);
                preview_nodes.insert(preview_node);
            }
            if let Some(&last_visible) = preview.nodes.last() {
                preview_hidden_counts.insert(last_visible, preview.hidden_count);
            }
        }
    }

    let visible_nodes = (0..node_count)
        .filter(|node_index| visible_set.contains(node_index))
        .collect();

    NeighborhoodProjection {
        visible_nodes,
        preview_nodes,
        preview_hidden_counts,
    }
}

struct PreviewBranch {
    nodes: Vec<usize>,
    hidden_count: usize,
}
