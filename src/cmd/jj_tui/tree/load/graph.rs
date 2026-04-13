mod roots;
mod traverse;

use super::super::TreeNode;
use ahash::{HashMap, HashSet};

pub(super) fn ordered_roots(
    commit_map: &HashMap<String, TreeNode>,
    children_map: &HashMap<String, Vec<String>>,
    working_copy_id: &str,
    base_id: Option<&str>,
) -> Vec<String> {
    roots::ordered_roots(commit_map, children_map, working_copy_id, base_id)
}

pub(super) fn build_nodes(
    commit_map: &HashMap<String, TreeNode>,
    children_map: &HashMap<String, Vec<String>>,
    roots: &[String],
) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let mut visited = HashSet::default();

    for root in roots {
        traverse::traverse(root, commit_map, children_map, &mut nodes, &mut visited, 0);
    }

    nodes
}
