use super::JjRepo;
use eyre::{Context, Result};
use jj_lib::commit::Commit;
use jj_lib::id_prefix::{IdPrefixContext, IdPrefixIndex};
use jj_lib::object_id::ObjectId;
use jj_lib::revset::{self, RevsetDiagnostics};

impl JjRepo {
    pub fn change_id_with_index(
        &self,
        index: &IdPrefixIndex,
        commit: &Commit,
        min_len: usize,
    ) -> Result<(String, usize)> {
        let unique_prefix_len = index
            .shortest_change_prefix_len(self.repo.as_ref(), commit.change_id())
            .wrap_err("failed to get shortest prefix length")?;
        let full_id = commit.change_id().reverse_hex();
        Ok((
            Self::prefix_display(&full_id, unique_prefix_len, min_len),
            unique_prefix_len,
        ))
    }

    pub fn commit_id_with_index(
        &self,
        index: &IdPrefixIndex,
        commit: &Commit,
        min_len: usize,
    ) -> Result<(String, usize)> {
        let unique_prefix_len = index
            .shortest_commit_prefix_len(self.repo.as_ref(), commit.id())
            .wrap_err("failed to get shortest commit prefix length")?;
        let full_id = commit.id().hex();
        Ok((
            Self::prefix_display(&full_id, unique_prefix_len, min_len),
            unique_prefix_len,
        ))
    }

    pub fn with_short_prefix_index<T>(
        &self,
        f: impl FnOnce(&IdPrefixIndex) -> Result<T>,
    ) -> Result<T> {
        self.with_revset_context("", |extensions, context| {
            let mut diagnostics = RevsetDiagnostics::new();
            let short_prefixes_revset =
                "present(@) | ancestors(immutable_heads()..) | present(trunk())";
            let disambiguate_expr = revset::parse(&mut diagnostics, short_prefixes_revset, context)
                .wrap_err("failed to parse short-prefixes revset")?;

            let id_prefix_context =
                IdPrefixContext::new(extensions.clone()).disambiguate_within(disambiguate_expr);
            let index = id_prefix_context
                .populate(self.repo.as_ref())
                .wrap_err("failed to populate id prefix index")?;

            f(&index)
        })
    }

    fn prefix_display(full_id: &str, unique_prefix_len: usize, min_len: usize) -> String {
        let display_len = unique_prefix_len.max(min_len).min(full_id.len());
        full_id[..display_len].to_string()
    }
}
