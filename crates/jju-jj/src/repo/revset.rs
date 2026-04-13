mod aliases;
mod query;

use super::JjRepo;
use eyre::Result;
use jj_lib::commit::Commit;
use jj_lib::ref_name::RemoteName;
use jj_lib::revset::{self, RevsetParseContext, RevsetWorkspaceContext};
use std::collections::HashMap;
use std::sync::Arc;

impl JjRepo {
    pub fn eval_revset(&self, revset_str: &str) -> Result<Vec<Commit>> {
        query::eval_revset(self, revset_str)
    }

    pub fn eval_revset_single(&self, revset_str: &str) -> Result<Commit> {
        query::eval_revset_single(self, revset_str)
    }

    pub fn working_copy_commit(&self) -> Result<Commit> {
        self.eval_revset_single("@")
    }

    pub(super) fn with_revset_context<T>(
        &self,
        user_email: &str,
        f: impl FnOnce(&Arc<revset::RevsetExtensions>, &RevsetParseContext<'_>) -> Result<T>,
    ) -> Result<T> {
        let aliases_map = aliases::aliases_map();
        let extensions = Arc::new(revset::RevsetExtensions::default());
        let path_converter = jj_lib::repo_path::RepoPathUiConverter::Fs {
            cwd: self.workspace.workspace_root().to_path_buf(),
            base: self.workspace.workspace_root().to_path_buf(),
        };
        let workspace_ctx = RevsetWorkspaceContext {
            path_converter: &path_converter,
            workspace_name: self.workspace.workspace_name(),
        };
        let context = RevsetParseContext {
            aliases_map: &aliases_map,
            local_variables: HashMap::new(),
            user_email,
            date_pattern_context: chrono::Utc::now().fixed_offset().into(),
            default_ignored_remote: Some(RemoteName::new("git")),
            workspace: Some(workspace_ctx),
            extensions: &extensions,
            use_glob_by_default: false,
        };

        f(&extensions, &context)
    }
}
