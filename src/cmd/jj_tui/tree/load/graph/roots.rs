use super::super::super::TreeNode;
use ahash::{HashMap, HashSet};

pub(super) fn ordered_roots(
    commit_map: &HashMap<String, TreeNode>,
    children_map: &HashMap<String, Vec<String>>,
    working_copy_id: &str,
    base_id: Option<&str>,
) -> Vec<String> {
    let revs_in_set: HashSet<&str> = commit_map.keys().map(String::as_str).collect();
    let mut roots: Vec<String> = commit_map
        .values()
        .filter(|commit| is_root(commit, &revs_in_set, base_id))
        .map(|commit| commit.change_id.clone())
        .collect();
    roots.sort();

    let working_copy_root = roots
        .iter()
        .find(|root| subtree_contains(root, working_copy_id, children_map))
        .cloned();
    let base_root = base_id.and_then(|id| roots.iter().find(|root| root.as_str() == id).cloned());

    let mut ordered = Vec::with_capacity(roots.len());
    if let Some(root) = working_copy_root.as_ref() {
        ordered.push(root.clone());
    }
    if let Some(root) = base_root.as_ref()
        && Some(root) != working_copy_root.as_ref()
    {
        ordered.push(root.clone());
    }
    for root in roots {
        if !ordered.contains(&root) {
            ordered.push(root);
        }
    }
    ordered
}

fn is_root(commit: &TreeNode, revs_in_set: &HashSet<&str>, base_id: Option<&str>) -> bool {
    if let Some(base_id) = base_id
        && commit.change_id == base_id
    {
        return true;
    }

    commit
        .parent_ids
        .iter()
        .all(|parent| !revs_in_set.contains(parent.as_str()))
}

fn subtree_contains(root: &str, target: &str, children_map: &HashMap<String, Vec<String>>) -> bool {
    if root == target {
        return true;
    }

    let mut stack = vec![root.to_string()];
    let mut visited = HashSet::default();
    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        if let Some(children) = children_map.get(&node) {
            for child in children {
                if child == target {
                    return true;
                }
                stack.push(child.clone());
            }
        }
    }
    false
}
