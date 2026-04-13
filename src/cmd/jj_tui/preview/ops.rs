mod moving;
mod rewire;

use super::{NodeId, PreviewRebaseType};
use crate::cmd::jj_tui::tree::TreeTopology;
use ahash::HashSet;

pub(super) struct RebasePreviewOp {
    pub source: NodeId,
    pub dest: NodeId,
    pub rebase_type: PreviewRebaseType,
    pub allow_branches: bool,
}

pub(super) struct OperationResult {
    pub topology: TreeTopology,
    pub moving_ids: HashSet<NodeId>,
}

pub(super) fn apply_rebase_preview(
    mut topology: TreeTopology,
    operation: RebasePreviewOp,
) -> OperationResult {
    let moving_ids = moving::moving_ids(&topology, &operation);

    if operation.source != operation.dest {
        rewire::apply(&mut topology, &operation, &moving_ids);
    }

    OperationResult {
        topology,
        moving_ids,
    }
}
