use super::TreeTopology;
use ahash::{HashSet, HashSetExt};

pub(super) fn descendants(topology: &TreeTopology, node_index: usize) -> HashSet<usize> {
    let mut result = HashSet::new();
    let mut stack = topology.children_of(node_index).to_vec();

    while let Some(current) = stack.pop() {
        if result.insert(current) {
            stack.extend(topology.children_of(current).iter().copied());
        }
    }

    result
}

pub(super) fn subtree_nodes_in_order(topology: &TreeTopology, root: usize) -> Vec<usize> {
    let mut nodes = Vec::new();
    collect_subtree(topology, root, &mut nodes);
    nodes
}

fn collect_subtree(topology: &TreeTopology, node_index: usize, nodes: &mut Vec<usize>) {
    nodes.push(node_index);
    for &child in topology.children_of(node_index) {
        collect_subtree(topology, child, nodes);
    }
}
