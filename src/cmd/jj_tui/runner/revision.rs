mod abandon;
mod edit;
mod rebase;

use super::{Effect, RunCtx};

pub(super) struct RevisionRunner<'a, 'b>(&'a mut RunCtx<'b>);

pub(super) fn handle(ctx: &mut RunCtx<'_>, effect: Effect) {
    let mut runner = RevisionRunner(ctx);

    match effect {
        Effect::RunEdit { rev } => runner.run_edit(&rev),
        Effect::RunNew { rev } => runner.run_new(&rev),
        Effect::RunCommit { message } => runner.run_commit(&message),
        Effect::RunAbandon { revset } => runner.run_abandon(&revset),
        Effect::RunRebase {
            source,
            dest,
            rebase_type,
            allow_branches,
        } => runner.run_rebase(&source, &dest, rebase_type, allow_branches),
        Effect::RunRebaseOntoTrunk {
            source,
            rebase_type,
        } => runner.run_rebase_onto_trunk(&source, rebase_type),
        Effect::RunUndo => runner.run_undo(),
        Effect::RunResolveDivergence {
            keep_commit_id,
            abandon_commit_ids,
        } => runner.run_resolve_divergence(&keep_commit_id, abandon_commit_ids),
        _ => unreachable!("unsupported revision effect: {effect:?}"),
    }
}

impl RevisionRunner<'_, '_> {
    pub(super) fn run_undo(&mut self) {
        match self.0.last_op.take() {
            Some(op_id) if !op_id.is_empty() => {
                match crate::cmd::jj_tui::commands::restore_op(&op_id) {
                    Ok(_) => self.0.success("Operation undone"),
                    Err(error) => self.0.error(format!("Undo failed: {error}")),
                }
            }
            _ => self.0.warn("Nothing to undo"),
        }
    }
}
