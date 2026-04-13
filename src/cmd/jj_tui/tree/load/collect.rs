use super::super::BookmarkInfo;
use super::super::{JjRepo, TreeNode};
use super::divergence::{build_divergent_commit_ids, divergent_versions_for_commit};
use super::identity::{build_change_id_display_map, parent_ids_for_commit};
use super::{CHANGE_ID_MIN_LEN, HashMap, IdPrefixIndex, Result};
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;

pub(super) struct TreeLoadInputs {
    pub(super) working_copy_id: String,
    pub(super) commit_map: HashMap<String, TreeNode>,
    pub(super) children_map: HashMap<String, Vec<String>>,
}

pub(super) fn collect_tree_inputs(
    jj_repo: &JjRepo,
    prefix_index: &IdPrefixIndex,
    commits: &[Commit],
    working_copy: &Commit,
) -> Result<TreeLoadInputs> {
    let working_copy_id = jj_repo
        .change_id_with_index(prefix_index, working_copy, CHANGE_ID_MIN_LEN)?
        .0;
    let bookmarks_by_commit = jj_repo.bookmarks_by_commit_id();
    let change_ids_by_full = build_change_id_display_map(jj_repo, prefix_index, commits)?;
    let divergent_commit_ids = build_divergent_commit_ids(commits);
    let mut parent_display_cache = HashMap::default();
    let mut commit_map: HashMap<String, TreeNode> = HashMap::default();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::default();

    for commit in commits {
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

    Ok(TreeLoadInputs {
        working_copy_id,
        commit_map,
        children_map,
    })
}
