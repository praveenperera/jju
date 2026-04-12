use super::JjRepo;
use ahash::HashMap;
use eyre::{Context, Result, bail};
use itertools::Itertools;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId;
use jj_lib::ref_name::{RemoteName, RemoteRefSymbol};
use jj_lib::repo::Repo;

impl JjRepo {
    pub fn bookmarks_by_commit_id(&self) -> HashMap<String, Vec<(String, bool)>> {
        let origin = RemoteName::new("origin");
        let mut bookmarks = HashMap::default();

        for (name, target) in self.repo.view().local_bookmarks() {
            let Some(commit_id) = target.as_resolved().and_then(|target| target.as_ref()) else {
                continue;
            };

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
                .map(|remote_id| remote_id != commit_id)
                .unwrap_or(false);

            bookmarks
                .entry(commit_id.hex())
                .or_insert_with(Vec::new)
                .push((name.as_str().to_string(), is_diverged));
        }

        bookmarks
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

    pub fn commit_by_id_hex(&self, commit_id_hex: &str) -> Result<Commit> {
        let Some(commit_id) = CommitId::try_from_hex(commit_id_hex) else {
            bail!("invalid commit id: {commit_id_hex}");
        };

        self.repo
            .store()
            .get_commit(&commit_id)
            .wrap_err_with(|| format!("failed to load commit {commit_id_hex}"))
    }

    /// Check if a commit has conflicts in its tree
    pub fn has_conflict(commit: &Commit) -> bool {
        commit.has_conflict()
    }
}
