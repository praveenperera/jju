use super::TreeTopology;
use crate::cmd::jj_tui::tree::TreeNode;
use ahash::HashSet;

pub(super) fn from_nodes(nodes: &[TreeNode]) -> TreeTopology {
    let mut parent = vec![None; nodes.len()];
    let mut children = vec![Vec::new(); nodes.len()];
    let mut roots = Vec::new();
    let mut depth_stack: Vec<(usize, usize)> = Vec::new();

    for (node_index, node) in nodes.iter().enumerate() {
        while let Some(&(parent_depth, _)) = depth_stack.last() {
            if parent_depth < node.depth {
                break;
            }
            depth_stack.pop();
        }

        if let Some((_, parent_index)) = depth_stack.last().copied() {
            parent[node_index] = Some(parent_index);
            children[parent_index].push(node_index);
        } else {
            roots.push(node_index);
        }

        depth_stack.push((node.depth, node_index));
    }

    TreeTopology {
        parent,
        children,
        roots,
    }
}

pub(super) fn project_visible(topology: &TreeTopology, visible_nodes: &[usize]) -> TreeTopology {
    let mut parent = vec![None; topology.parent.len()];
    let mut children = vec![Vec::new(); topology.children.len()];
    let mut roots = Vec::new();
    let visible_set: HashSet<usize> = visible_nodes.iter().copied().collect();

    for &node_index in visible_nodes {
        let mut current_parent = topology.parent_of(node_index);
        while let Some(parent_index) = current_parent {
            if visible_set.contains(&parent_index) {
                parent[node_index] = Some(parent_index);
                children[parent_index].push(node_index);
                break;
            }
            current_parent = topology.parent_of(parent_index);
        }

        if parent[node_index].is_none() {
            roots.push(node_index);
        }
    }

    TreeTopology {
        parent,
        children,
        roots,
    }
}
