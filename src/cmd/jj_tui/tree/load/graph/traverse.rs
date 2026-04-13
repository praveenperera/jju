use super::super::super::TreeNode;
use ahash::{HashMap, HashSet};

pub(super) fn traverse(
    change_id: &str,
    commit_map: &HashMap<String, TreeNode>,
    children_map: &HashMap<String, Vec<String>>,
    nodes: &mut Vec<TreeNode>,
    visited: &mut HashSet<String>,
    depth: usize,
) {
    if !visited.insert(change_id.to_string()) {
        return;
    }

    let Some(node) = commit_map.get(change_id) else {
        return;
    };

    let mut node = node.clone();
    node.depth = depth;
    nodes.push(node);

    let Some(children) = children_map.get(change_id) else {
        return;
    };

    let mut sorted_children = children.clone();
    sorted_children.sort();
    for child in sorted_children {
        traverse(&child, commit_map, children_map, nodes, visited, depth + 1);
    }
}
