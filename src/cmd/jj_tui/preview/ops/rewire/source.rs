use super::{RebasePreviewOp, child_ids};
use crate::cmd::jj_tui::preview::{NodeId, PreviewRebaseType};
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) fn reparent_source_children(
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
