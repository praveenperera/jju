use super::super::super::TreeTopology;
use super::PreviewBranch;

pub(super) fn preview_branch(
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
