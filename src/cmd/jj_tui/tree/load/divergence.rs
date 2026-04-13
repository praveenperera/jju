use super::super::DivergentVersion;
use ahash::HashMap;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;

pub(super) fn build_divergent_commit_ids(commits: &[Commit]) -> HashMap<String, Vec<String>> {
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

pub(super) fn divergent_versions_for_commit(
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
