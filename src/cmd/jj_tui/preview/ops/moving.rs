use super::RebasePreviewOp;
use crate::cmd::jj_tui::preview::NodeId;
use crate::cmd::jj_tui::preview::PreviewRebaseType;
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) fn moving_ids(topology: &TreeTopology, operation: &RebasePreviewOp) -> HashSet<NodeId> {
    let mut moving_ids = HashSet::default();
    moving_ids.insert(operation.source);

    if operation.rebase_type == PreviewRebaseType::WithDescendants {
        moving_ids.extend(
            topology
                .descendants(operation.source.0)
                .into_iter()
                .map(NodeId),
        );
    }

    moving_ids
}
