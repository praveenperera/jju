use super::{DisplaySlot, NodeId, NodeRole};
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) fn identity_slots(
    visible_nodes: &[usize],
    visual_depths: &[usize],
    source: NodeId,
    dest: NodeId,
) -> Vec<DisplaySlot> {
    visible_nodes
        .iter()
        .zip(visual_depths)
        .map(|(&node_index, &visual_depth)| DisplaySlot {
            node_id: NodeId(node_index),
            visual_depth,
            role: slot_role(NodeId(node_index), &HashSet::default(), source, dest),
        })
        .collect()
}

pub(super) fn project_slots(
    topology: &TreeTopology,
    moving_ids: &HashSet<NodeId>,
    source: NodeId,
    dest: NodeId,
) -> Vec<DisplaySlot> {
    let mut traversal = SlotTraversal {
        moving_ids,
        source,
        dest,
        slots: Vec::new(),
        visited: HashSet::default(),
    };

    for &root in topology.roots() {
        dfs_traverse(topology, NodeId(root), 0, &mut traversal);
    }

    traversal.slots
}

fn dfs_traverse(
    topology: &TreeTopology,
    node_id: NodeId,
    depth: usize,
    traversal: &mut SlotTraversal<'_>,
) {
    if !traversal.visited.insert(node_id) {
        return;
    }

    traversal.slots.push(DisplaySlot {
        node_id,
        visual_depth: depth,
        role: slot_role(
            node_id,
            traversal.moving_ids,
            traversal.source,
            traversal.dest,
        ),
    });

    for &child in topology.children_of(node_id.0) {
        dfs_traverse(topology, NodeId(child), depth + 1, traversal);
    }
}

fn slot_role(
    node_id: NodeId,
    moving_ids: &HashSet<NodeId>,
    source: NodeId,
    dest: NodeId,
) -> NodeRole {
    if node_id == source {
        NodeRole::Source
    } else if node_id == dest {
        NodeRole::Destination
    } else if moving_ids.contains(&node_id) {
        NodeRole::Moving
    } else {
        NodeRole::Normal
    }
}

struct SlotTraversal<'a> {
    moving_ids: &'a HashSet<NodeId>,
    source: NodeId,
    dest: NodeId,
    slots: Vec<DisplaySlot>,
    visited: HashSet<NodeId>,
}
