use super::{RebasePreviewOp, child_ids, last_moving_descendant};
use crate::cmd::jj_tui::preview::NodeId;
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) fn reparent_dest_children(
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
