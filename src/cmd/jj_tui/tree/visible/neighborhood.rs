use super::super::{NeighborhoodEntry, TreeTopology, VisibleEntry};
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

    let projection = neighborhood_projection(
        node_count,
        topology,
        anchor_index,
        ancestor_limit,
        preview_depth_limit,
    );
    projected_entries(topology, projection)
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
    let mainline = mainline_path(topology, anchor_index, ancestor_limit);
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

            let preview = preview_branch(topology, child_index, preview_depth_limit);
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

fn mainline_path(
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
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
    loop {
        let [child_index] = topology.children_of(current) else {
            break;
        };
        mainline.push(*child_index);
        current = *child_index;
    }

    mainline
}

struct PreviewBranch {
    nodes: Vec<usize>,
    hidden_count: usize,
}

fn preview_branch(
    topology: &TreeTopology,
    branch_root: usize,
    preview_depth_limit: usize,
) -> PreviewBranch {
    let mut nodes = vec![branch_root];
    let mut current = branch_root;
    let mut remaining = preview_depth_limit.saturating_sub(1);

    while remaining > 0 {
        let [next_child] = topology.children_of(current) else {
            break;
        };
        nodes.push(*next_child);
        current = *next_child;
        remaining -= 1;
    }

    let hidden_count = topology
        .subtree_nodes_in_order(branch_root)
        .len()
        .saturating_sub(nodes.len());

    PreviewBranch {
        nodes,
        hidden_count,
    }
}

fn projected_entries(
    topology: &TreeTopology,
    projection: NeighborhoodProjection,
) -> Vec<VisibleEntry> {
    let projected = topology.project_visible(&projection.visible_nodes);
    let mut visual_depths: HashMap<usize, usize> = HashMap::default();
    let mut entries = Vec::with_capacity(projection.visible_nodes.len());
    let mut seen_root = false;

    for &node_index in &projection.visible_nodes {
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
            neighborhood: projection.preview_nodes.contains(&node_index).then(|| {
                NeighborhoodEntry {
                    is_preview: true,
                    hidden_count: projection
                        .preview_hidden_counts
                        .get(&node_index)
                        .copied()
                        .unwrap_or_default(),
                }
            }),
        });
    }

    entries
}
