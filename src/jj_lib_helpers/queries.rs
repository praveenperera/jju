use super::JjRepo;
use eyre::{Context, Result};
use itertools::Itertools;
use jj_lib::commit::Commit;
use jj_lib::ref_name::{RemoteName, RemoteRefSymbol};
use jj_lib::repo::Repo;

impl JjRepo {
    /// Get local bookmarks on a specific commit with divergence status
    /// A bookmark is diverged if local differs from origin
    pub fn bookmarks_with_state(&self, commit: &Commit) -> Vec<(String, bool)> {
        let origin = RemoteName::new("origin");
        self.repo
            .view()
            .local_bookmarks()
            .filter_map(|(name, target)| {
                let resolved = target.as_resolved().and_then(|target| target.as_ref());
                if resolved != Some(commit.id()) {
                    return None;
                }

                let symbol = RemoteRefSymbol {
                    name,
                    remote: origin,
                };
                let is_diverged = self
                    .repo
                    .view()
                    .get_remote_bookmark(symbol)
                    .target
                    .as_resolved()
                    .and_then(|target| target.as_ref())
                    .map(|remote_id| remote_id != commit.id())
                    .unwrap_or(false);

                Some((name.as_str().to_string(), is_diverged))
            })
            .collect()
    }

    /// Get all local bookmark names in the repository
    pub fn all_local_bookmarks(&self) -> Vec<String> {
        self.repo
            .view()
            .local_bookmarks()
            .map(|(name, _target)| name.as_str().to_string())
            .collect()
    }

    /// Get parent commits for a commit
    pub fn parent_commits(&self, commit: &Commit) -> Result<Vec<Commit>> {
        commit
            .parents()
            .try_collect()
            .wrap_err("failed to get parent commits")
    }

    /// Check if a commit has conflicts in its tree
    pub fn has_conflict(commit: &Commit) -> bool {
        commit.has_conflict()
    }

    /// Check if a commit is divergent (same change_id, multiple visible commits)
    pub fn is_commit_divergent(&self, commit: &Commit) -> bool {
        self.repo
            .resolve_change_id(commit.change_id())
            .ok()
            .flatten()
            .map(|targets| targets.is_divergent())
            .unwrap_or(false)
    }

    /// Get all visible commit IDs for a divergent change_id
    /// Returns empty vec if not divergent (single commit for change_id)
    pub fn get_divergent_commit_ids(&self, commit: &Commit) -> Vec<jj_lib::backend::CommitId> {
        let Some(targets) = self
            .repo
            .resolve_change_id(commit.change_id())
            .ok()
            .flatten()
        else {
            return Vec::new();
        };
        if targets.is_divergent() {
            targets
                .visible_with_offsets()
                .map(|(_, commit_id)| commit_id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get divergent commits for a change_id, sorted by timestamp (newest first)
    pub fn get_divergent_commits(&self, commit: &Commit) -> Result<Vec<Commit>> {
        let commit_ids = self.get_divergent_commit_ids(commit);
        if commit_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut commits: Vec<Commit> = commit_ids
            .into_iter()
            .filter_map(|commit_id| self.repo.store().get_commit(&commit_id).ok())
            .collect();

        commits.sort_by(|left, right| {
            let left_ts = left.author().timestamp.timestamp.0;
            let right_ts = right.author().timestamp.timestamp.0;
            right_ts.cmp(&left_ts)
        });

        Ok(commits)
    }
}
