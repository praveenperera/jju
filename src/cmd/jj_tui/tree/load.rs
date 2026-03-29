use super::{BookmarkInfo, DivergentVersion, JjRepo, TreeNode, TreeState};
use ahash::{HashMap, HashSet};
use eyre::Result;
use jj_lib::object_id::ObjectId;

pub(super) fn load_tree_state(jj_repo: &JjRepo, base: &str) -> Result<TreeState> {
    let working_copy = jj_repo.working_copy_commit()?;
    let working_copy_id = jj_repo.shortest_change_id(&working_copy, 4)?;

    let revset = format!("{base} | ancestors(immutable_heads().., 2) | @::");
    let commits = jj_repo.eval_revset(&revset)?;

    let mut commit_map: HashMap<String, TreeNode> = HashMap::default();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::default();

    for commit in &commits {
        let (change_id, unique_prefix_len) = jj_repo.change_id_with_prefix_len(commit, 4)?;
        let (commit_id, unique_commit_prefix_len) = jj_repo.commit_id_with_prefix_len(commit, 7)?;
        let bookmarks: Vec<BookmarkInfo> = jj_repo
            .bookmarks_with_state(commit)
            .into_iter()
            .map(|(name, is_diverged)| BookmarkInfo { name, is_diverged })
            .collect();
        let description = JjRepo::description_first_line(commit);
        let full_description = commit.description().to_string();

        let parents = jj_repo.parent_commits(commit)?;
        let parent_ids: Vec<String> = parents
            .iter()
            .filter_map(|parent| jj_repo.shortest_change_id(parent, 4).ok())
            .collect();

        let is_working_copy = change_id == working_copy_id;
        let has_conflicts = JjRepo::has_conflict(commit);
        let author_name = JjRepo::author_name(commit);
        let author_email = JjRepo::author_email(commit);
        let timestamp = JjRepo::author_timestamp_relative(commit);

        let is_divergent = jj_repo.is_commit_divergent(commit);
        let divergent_versions = divergent_versions(jj_repo, commit, is_working_copy, is_divergent);

        let node = TreeNode {
            change_id: change_id.clone(),
            unique_prefix_len,
            commit_id,
            unique_commit_prefix_len,
            description,
            full_description,
            bookmarks,
            is_working_copy,
            has_conflicts,
            is_divergent,
            divergent_versions,
            parent_ids: parent_ids.clone(),
            depth: 0,
            author_name,
            author_email,
            timestamp,
        };

        commit_map.insert(change_id.clone(), node);
        for parent_id in parent_ids {
            children_map
                .entry(parent_id)
                .or_default()
                .push(change_id.clone());
        }
    }

    if commit_map.is_empty() {
        return Ok(TreeState::empty());
    }

    let base_id = jj_repo
        .eval_revset_single(base)
        .ok()
        .and_then(|commit| jj_repo.shortest_change_id(&commit, 4).ok());
    let roots = ordered_roots(
        &commit_map,
        &children_map,
        &working_copy_id,
        base_id.as_deref(),
    );
    let nodes = build_nodes(&commit_map, &children_map, &roots);

    Ok(TreeState::from_nodes(nodes))
}

fn divergent_versions(
    jj_repo: &JjRepo,
    commit: &jj_lib::commit::Commit,
    is_working_copy: bool,
    is_divergent: bool,
) -> Vec<DivergentVersion> {
    if !is_divergent {
        return Vec::new();
    }

    jj_repo
        .get_divergent_commits(commit)
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, divergent_commit)| {
            let commit_id = divergent_commit.id().hex();
            let is_local = index == 0 || commit_id == commit.id().hex() && is_working_copy;
            DivergentVersion {
                commit_id,
                is_local,
            }
        })
        .collect()
}

fn ordered_roots(
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

fn build_nodes(
    commit_map: &HashMap<String, TreeNode>,
    children_map: &HashMap<String, Vec<String>>,
    roots: &[String],
) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let mut visited = HashSet::default();

    for root in roots {
        traverse(root, commit_map, children_map, &mut nodes, &mut visited, 0);
    }

    nodes
}

fn traverse(
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
