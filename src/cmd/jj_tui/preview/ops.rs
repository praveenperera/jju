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

    if operation.source != operation.dest {
        reparent_source_children(&mut topology, &operation, &moving_ids);
        reparent_dest_children(&mut topology, &operation, &moving_ids);
        topology.remove_from_parent(operation.source.0);
        topology.add_child(operation.dest.0, operation.source.0);
    }

    OperationResult {
        topology,
        moving_ids,
    }
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
    let source_children: Vec<NodeId> = topology
        .children_of(operation.source.0)
        .iter()
        .copied()
        .map(NodeId)
        .collect();

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

    let dest_children: Vec<NodeId> = topology
        .children_of(operation.dest.0)
        .iter()
        .copied()
        .map(NodeId)
        .collect();
    if dest_children.is_empty() {
        return;
    }

    let last_moving = find_last_descendant(topology, operation.source, moving_ids);
    for child in dest_children {
        if moving_ids.contains(&child) {
            continue;
        }

        topology.remove_from_parent(child.0);
        topology.add_child(last_moving.0, child.0);
    }
}

fn find_last_descendant(
    topology: &TreeTopology,
    start: NodeId,
    moving_ids: &HashSet<NodeId>,
) -> NodeId {
    let mut current = start;
    loop {
        let moving_children: Vec<NodeId> = topology
            .children_of(current.0)
            .iter()
            .copied()
            .map(NodeId)
            .filter(|child| moving_ids.contains(child))
            .collect();

        if moving_children.is_empty() {
            return current;
        }

        current = moving_children[0];
    }
}
