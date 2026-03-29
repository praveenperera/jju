use super::JjRepo;
use eyre::{Context, Result, bail};
use itertools::Itertools;
use jj_lib::commit::Commit;
use jj_lib::id_prefix::IdPrefixContext;
use jj_lib::ref_name::RemoteName;
use jj_lib::repo::Repo;
use jj_lib::revset::{
    self, RevsetDiagnostics, RevsetIteratorExt, RevsetParseContext, RevsetWorkspaceContext,
    SymbolResolver,
};
use std::collections::HashMap;
use std::sync::Arc;

impl JjRepo {
    /// Evaluate a revset string and return matching commits
    pub fn eval_revset(&self, revset_str: &str) -> Result<Vec<Commit>> {
        self.with_revset_context("jj-lib@localhost", |extensions, context| {
            let mut diagnostics = RevsetDiagnostics::new();
            let expression = revset::parse(&mut diagnostics, revset_str, context)
                .wrap_err_with(|| format!("failed to parse revset: {revset_str}"))?;

            let id_prefix_context = IdPrefixContext::default();
            let symbol_resolver =
                SymbolResolver::new(self.repo.as_ref(), extensions.symbol_resolvers())
                    .with_id_prefix_context(&id_prefix_context);

            let resolved = expression
                .resolve_user_expression(self.repo.as_ref(), &symbol_resolver)
                .wrap_err("failed to resolve revset expression")?;

            let evaluated = resolved
                .evaluate(self.repo.as_ref())
                .wrap_err("failed to evaluate revset")?;

            evaluated
                .iter()
                .commits(self.repo.store())
                .try_collect()
                .wrap_err("failed to collect commits")
        })
    }

    /// Evaluate a revset and return a single commit (error if 0 or >1 results)
    pub fn eval_revset_single(&self, revset_str: &str) -> Result<Commit> {
        let commits = self.eval_revset(revset_str)?;
        let mut commits = commits.into_iter();

        match (commits.next(), commits.next()) {
            (None, _) => bail!("revset '{}' matched no commits", revset_str),
            (Some(commit), None) => Ok(commit),
            (Some(_), Some(_)) => bail!(
                "revset '{}' matched {} commits, expected 1",
                revset_str,
                commits.count() + 2
            ),
        }
    }

    /// Get the working copy commit
    pub fn working_copy_commit(&self) -> Result<Commit> {
        self.eval_revset_single("@")
    }

    /// Get the revset aliases map with jj's default aliases
    fn aliases_map(&self) -> revset::RevsetAliasesMap {
        let mut aliases_map = revset::RevsetAliasesMap::new();

        let default_aliases = [
            (
                "trunk()",
                r#"latest(
              remote_bookmarks(exact:"main", exact:"origin") |
              remote_bookmarks(exact:"master", exact:"origin") |
              remote_bookmarks(exact:"trunk", exact:"origin") |
              remote_bookmarks(exact:"main", exact:"upstream") |
              remote_bookmarks(exact:"master", exact:"upstream") |
              remote_bookmarks(exact:"trunk", exact:"upstream") |
              root()
            )"#,
            ),
            (
                "builtin_immutable_heads()",
                "trunk() | tags() | untracked_remote_bookmarks()",
            ),
            ("immutable_heads()", "builtin_immutable_heads()"),
            ("immutable()", "::(immutable_heads() | root())"),
            ("mutable()", "~immutable()"),
        ];

        for (name, definition) in default_aliases {
            let _ = aliases_map.insert(name, definition);
        }
        aliases_map
    }

    pub(super) fn with_revset_context<T>(
        &self,
        user_email: &str,
        f: impl FnOnce(&Arc<revset::RevsetExtensions>, &RevsetParseContext<'_>) -> Result<T>,
    ) -> Result<T> {
        let aliases_map = self.aliases_map();
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
