use super::{BookmarkInfo, DivergentVersion, JjRepo, TreeLoadScope, TreeNode, TreeState};
use ahash::{HashMap, HashSet};
use eyre::Result;
use jj_lib::commit::Commit;
use jj_lib::id_prefix::IdPrefixIndex;
use jj_lib::object_id::ObjectId;
use log::info;
use std::time::Instant;

const CHANGE_ID_MIN_LEN: usize = 4;

pub(super) fn load_tree_state(
    jj_repo: &JjRepo,
    base: &str,
    load_scope: TreeLoadScope,
) -> Result<TreeState> {
    let started_at = Instant::now();
    let working_copy = jj_repo.working_copy_commit()?;
    let revset = revset_for_scope(base, load_scope);
    let commits = jj_repo.eval_revset(&revset)?;

    if commits.is_empty() {
        return Ok(TreeState::empty(load_scope));
    }

    jj_repo.with_short_prefix_index(|prefix_index| {
        let working_copy_id = jj_repo
            .change_id_with_index(prefix_index, &working_copy, CHANGE_ID_MIN_LEN)?
            .0;
        let bookmarks_by_commit = jj_repo.bookmarks_by_commit_id();
        let change_ids_by_full = build_change_id_display_map(jj_repo, prefix_index, &commits)?;
        let divergent_commit_ids = build_divergent_commit_ids(&commits);
        let mut parent_display_cache = HashMap::default();
        let mut commit_map: HashMap<String, TreeNode> = HashMap::default();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::default();

        for commit in &commits {
            let full_change_id = commit.change_id().reverse_hex();
            let (change_id, unique_prefix_len) = change_ids_by_full
                .get(&full_change_id)
                .cloned()
                .unwrap_or_else(|| (full_change_id.clone(), CHANGE_ID_MIN_LEN));
            let commit_id = commit.id().hex();
            let bookmarks = bookmarks_by_commit
                .get(commit_id.as_str())
                .cloned()
                .unwrap_or_default();
            let parent_ids = parent_ids_for_commit(
                jj_repo,
                prefix_index,
                commit,
                &change_ids_by_full,
                &mut parent_display_cache,
            )?;
            let is_working_copy = change_id == working_copy_id;
            let is_divergent = divergent_commit_ids.contains_key(&full_change_id);
            let divergent_versions = divergent_versions_for_commit(
                &divergent_commit_ids,
                &full_change_id,
                &commit_id,
                is_working_copy,
            );

            let node = TreeNode {
                change_id: change_id.clone(),
                unique_prefix_len,
                commit_id,
                description: JjRepo::description_first_line(commit),
                bookmarks: bookmarks
                    .into_iter()
                    .map(|(name, is_diverged)| BookmarkInfo { name, is_diverged })
                    .collect(),
                is_working_copy,
                has_conflicts: JjRepo::has_conflict(commit),
                is_divergent,
                divergent_versions,
                parent_ids: parent_ids.clone(),
                depth: 0,
                details: None,
            };

            commit_map.insert(change_id.clone(), node);
            for parent_id in parent_ids {
                children_map
                    .entry(parent_id)
                    .or_default()
                    .push(change_id.clone());
            }
        }

        let base_id = jj_repo.eval_revset_single(base).ok().and_then(|commit| {
            jj_repo
                .change_id_with_index(prefix_index, &commit, CHANGE_ID_MIN_LEN)
                .ok()
                .map(|(change_id, _)| change_id)
        });
        let roots = ordered_roots(
            &commit_map,
            &children_map,
            &working_copy_id,
            base_id.as_deref(),
        );
        let nodes = build_nodes(&commit_map, &children_map, &roots);

        info!(
            "Loaded tree summary for {} commits in {:?}",
            commits.len(),
            started_at.elapsed()
        );

        Ok(TreeState::from_nodes(nodes, load_scope))
    })
}

fn revset_for_scope(base: &str, load_scope: TreeLoadScope) -> String {
    match load_scope {
        TreeLoadScope::Stack => format!("{base} | ancestors(immutable_heads().., 2) | @::"),
        TreeLoadScope::Neighborhood => format!("{base} | ancestors(immutable_heads()..) | @::"),
    }
}

fn build_change_id_display_map(
    jj_repo: &JjRepo,
    prefix_index: &IdPrefixIndex,
    commits: &[Commit],
) -> Result<HashMap<String, (String, usize)>> {
    let mut change_ids = HashMap::default();

    for commit in commits {
        let full_change_id = commit.change_id().reverse_hex();
        let display = jj_repo.change_id_with_index(prefix_index, commit, CHANGE_ID_MIN_LEN)?;
        change_ids.insert(full_change_id, display);
    }

    Ok(change_ids)
}

#[cfg(test)]
mod tests {
    use super::revset_for_scope;
    use crate::cmd::jj_tui::tree::TreeLoadScope;

    #[test]
    fn stack_scope_keeps_capped_revset() {
        assert_eq!(
            revset_for_scope("trunk()", TreeLoadScope::Stack),
            "trunk() | ancestors(immutable_heads().., 2) | @::"
        );
    }

    #[test]
    fn neighborhood_scope_uses_uncapped_revset() {
        assert_eq!(
            revset_for_scope("trunk()", TreeLoadScope::Neighborhood),
            "trunk() | ancestors(immutable_heads()..) | @::"
        );
    }
}

fn build_divergent_commit_ids(commits: &[Commit]) -> HashMap<String, Vec<String>> {
    let mut divergent_by_change = HashMap::<String, Vec<&Commit>>::default();

    for commit in commits {
        divergent_by_change
            .entry(commit.change_id().reverse_hex())
            .or_default()
            .push(commit);
    }

    divergent_by_change
        .into_iter()
        .filter_map(|(change_id, mut commits)| {
            if commits.len() < 2 {
                return None;
            }

            commits.sort_by(|left, right| {
                let left_ts = left.author().timestamp.timestamp.0;
                let right_ts = right.author().timestamp.timestamp.0;
                right_ts.cmp(&left_ts)
            });

            Some((
                change_id,
                commits
                    .into_iter()
                    .map(|commit| commit.id().hex())
                    .collect::<Vec<_>>(),
            ))
        })
        .collect()
}

fn divergent_versions_for_commit(
    divergent_commit_ids: &HashMap<String, Vec<String>>,
    full_change_id: &str,
    commit_id: &str,
    is_working_copy: bool,
) -> Vec<DivergentVersion> {
    divergent_commit_ids
        .get(full_change_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, divergent_commit_id)| DivergentVersion {
            is_local: index == 0 || (is_working_copy && divergent_commit_id == commit_id),
            commit_id: divergent_commit_id,
        })
        .collect()
}

fn parent_ids_for_commit(
    jj_repo: &JjRepo,
    prefix_index: &IdPrefixIndex,
    commit: &Commit,
    change_ids_by_full: &HashMap<String, (String, usize)>,
    parent_display_cache: &mut HashMap<String, String>,
) -> Result<Vec<String>> {
    let parents = jj_repo.parent_commits(commit)?;
    let mut parent_ids = Vec::with_capacity(parents.len());

    for parent in parents {
        let full_change_id = parent.change_id().reverse_hex();

        if let Some((display, _)) = change_ids_by_full.get(&full_change_id) {
            parent_ids.push(display.clone());
            continue;
        }

        if let Some(display) = parent_display_cache.get(&full_change_id) {
            parent_ids.push(display.clone());
            continue;
        }

        let (display, _) =
            jj_repo.change_id_with_index(prefix_index, &parent, CHANGE_ID_MIN_LEN)?;
        parent_display_cache.insert(full_change_id, display.clone());
        parent_ids.push(display);
    }

    Ok(parent_ids)
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
