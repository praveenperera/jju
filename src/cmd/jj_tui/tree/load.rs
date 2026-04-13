mod divergence;
mod graph;
mod identity;

use super::{BookmarkInfo, JjRepo, TreeLoadScope, TreeNode, TreeState};
use ahash::HashMap;
use divergence::{build_divergent_commit_ids, divergent_versions_for_commit};
use eyre::Result;
use graph::{build_nodes, ordered_roots};
use identity::{build_change_id_display_map, parent_ids_for_commit};
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
