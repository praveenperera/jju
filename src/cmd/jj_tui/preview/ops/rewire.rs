use super::RebasePreviewOp;
use crate::cmd::jj_tui::preview::{NodeId, PreviewRebaseType};
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) fn apply(
    topology: &mut TreeTopology,
    operation: &RebasePreviewOp,
    moving_ids: &HashSet<NodeId>,
) {
    reparent_source_children(topology, operation, moving_ids);
    reparent_dest_children(topology, operation, moving_ids);
    topology.remove_from_parent(operation.source.0);
    topology.add_child(operation.dest.0, operation.source.0);
}

fn reparent_source_children(
    topology: &mut TreeTopology,
    operation: &RebasePreviewOp,
    moving_ids: &HashSet<NodeId>,
) {
    if operation.rebase_type != PreviewRebaseType::Single {
        return;
    }

    let source_parent = topology.parent_of(operation.source.0).map(NodeId);
    let source_children = child_ids(topology, operation.source);

    for child in source_children {
        if moving_ids.contains(&child) {
            continue;
        }

        topology.remove_from_parent(child.0);
        if let Some(parent) = source_parent {
            topology.add_child(parent.0, child.0);
        }
    }
}

fn reparent_dest_children(
    topology: &mut TreeTopology,
    operation: &RebasePreviewOp,
    moving_ids: &HashSet<NodeId>,
) {
    if operation.allow_branches {
        return;
    }

    let dest_children = child_ids(topology, operation.dest);
    if dest_children.is_empty() {
        return;
    }

    let last_moving = last_moving_descendant(topology, operation.source, moving_ids);
    for child in dest_children {
        if moving_ids.contains(&child) {
            continue;
        }

        topology.remove_from_parent(child.0);
        topology.add_child(last_moving.0, child.0);
    }
}

fn child_ids(topology: &TreeTopology, node_id: NodeId) -> Vec<NodeId> {
    topology
        .children_of(node_id.0)
        .iter()
        .copied()
        .map(NodeId)
        .collect()
}

fn last_moving_descendant(
    topology: &TreeTopology,
    start: NodeId,
    moving_ids: &HashSet<NodeId>,
) -> NodeId {
    let mut current = start;
    loop {
        let moving_children: Vec<NodeId> = child_ids(topology, current)
            .into_iter()
            .filter(|child| moving_ids.contains(child))
            .collect();

        if moving_children.is_empty() {
            return current;
        }

        current = moving_children[0];
    }
}
