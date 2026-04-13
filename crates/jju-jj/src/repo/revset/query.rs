use super::super::JjRepo;
use eyre::{Context, Result, bail};
use itertools::Itertools;
use jj_lib::commit::Commit;
use jj_lib::id_prefix::IdPrefixContext;
use jj_lib::repo::Repo;
use jj_lib::revset::{self, RevsetDiagnostics, RevsetIteratorExt, SymbolResolver};

pub(super) fn eval_revset(repo: &JjRepo, revset_str: &str) -> Result<Vec<Commit>> {
    repo.with_revset_context("jj-lib@localhost", |extensions, context| {
        let mut diagnostics = RevsetDiagnostics::new();
        let expression = revset::parse(&mut diagnostics, revset_str, context)
            .wrap_err_with(|| format!("failed to parse revset: {revset_str}"))?;

        let id_prefix_context = IdPrefixContext::default();
        let symbol_resolver =
            SymbolResolver::new(repo.repo.as_ref(), extensions.symbol_resolvers())
                .with_id_prefix_context(&id_prefix_context);

        let resolved = expression
            .resolve_user_expression(repo.repo.as_ref(), &symbol_resolver)
            .wrap_err("failed to resolve revset expression")?;

        let evaluated = resolved
            .evaluate(repo.repo.as_ref())
            .wrap_err("failed to evaluate revset")?;

        evaluated
            .iter()
            .commits(repo.repo.store())
            .try_collect()
            .wrap_err("failed to collect commits")
    })
}

pub(super) fn eval_revset_single(repo: &JjRepo, revset_str: &str) -> Result<Commit> {
    let commits = eval_revset(repo, revset_str)?;
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
