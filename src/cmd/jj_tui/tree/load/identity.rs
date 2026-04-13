use super::{CHANGE_ID_MIN_LEN, HashMap, IdPrefixIndex, JjRepo, Result};
use jj_lib::commit::Commit;

pub(super) fn build_change_id_display_map(
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

pub(super) fn parent_ids_for_commit(
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
