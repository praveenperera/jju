mod destination;
mod source;

use super::RebasePreviewOp;
use crate::cmd::jj_tui::preview::NodeId;
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) fn apply(
    topology: &mut TreeTopology,
    operation: &RebasePreviewOp,
    moving_ids: &HashSet<NodeId>,
) {
    source::reparent_source_children(topology, operation, moving_ids);
    destination::reparent_dest_children(topology, operation, moving_ids);
    attach_source_at_destination(topology, operation);
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

fn attach_source_at_destination(topology: &mut TreeTopology, operation: &RebasePreviewOp) {
    topology.remove_from_parent(operation.source.0);
    topology.add_child(operation.dest.0, operation.source.0);
}
