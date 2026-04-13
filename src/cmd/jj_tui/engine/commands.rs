mod divergence;
mod git;
mod pr;
mod revision;

use super::{Action, Effect, ReduceCtx};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EditWorkingCopy => revision::edit_working_copy(ctx),
        Action::CreateNewCommit => revision::create_new_commit(ctx),
        Action::CommitWorkingCopy => revision::commit_working_copy(ctx),
        Action::EditDescription => revision::edit_description(ctx),
        Action::Undo => {
            ctx.effects.push(Effect::RunUndo);
            ctx.effects.push(Effect::RefreshTree);
        }
        Action::GitFetch => git::run_simple_refresh(ctx, Effect::RunGitFetch),
        Action::GitImport => git::run_simple_refresh(ctx, Effect::RunGitImport),
        Action::GitExport => git::run_simple_refresh(ctx, Effect::RunGitExport),
        Action::ResolveDivergence => divergence::resolve_divergence(ctx),
        Action::CreatePR => pr::create_pr(ctx),
        _ => unreachable!("unsupported command action: {action:?}"),
    }
}
