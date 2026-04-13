use super::{Effect, RunCtx};
use crate::cmd::jj_tui::runner::{error, operations};
use crate::cmd::jj_tui::state::{MessageKind, RebaseType};

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
    pub(super) fn run_edit(&mut self, rev: &str) {
        match crate::cmd::jj_tui::commands::revision::edit(rev) {
            Ok(_) => self.0.success(format!("Now editing {rev}")),
            Err(error) => self.0.error(format!("Edit failed: {error}")),
        }
    }

    pub(super) fn run_new(&mut self, rev: &str) {
        match crate::cmd::jj_tui::commands::revision::new(rev) {
            Ok(_) => self.0.success("Created new commit"),
            Err(error) => self.0.error(format!("Failed: {error}")),
        }
    }

    pub(super) fn run_commit(&mut self, message: &str) {
        match crate::cmd::jj_tui::commands::revision::commit(message) {
            Ok(_) => self.0.success("Changes committed"),
            Err(error) => self.0.error(format!("Commit failed: {error}")),
        }
    }

    pub(super) fn run_abandon(&mut self, revset: &str) {
        match crate::cmd::jj_tui::commands::revision::abandon(revset) {
            Ok(_) => {
                let count = revset.matches('|').count() + 1;
                if count == 1 {
                    self.0.success("Revision abandoned");
                } else {
                    self.0.success(format!("{count} revisions abandoned"));
                }
            }
            Err(error_value) => {
                let details = format!("{error_value}");
                self.0.set_status(
                    error::set_error_with_details("Abandon failed", &details),
                    MessageKind::Error,
                );
            }
        }
    }

    pub(super) fn run_resolve_divergence(
        &mut self,
        keep_commit_id: &str,
        abandon_commit_ids: Vec<String>,
    ) {
        let revset = abandon_commit_ids.join(" | ");
        match crate::cmd::jj_tui::commands::revision::abandon(&revset) {
            Ok(_) => {
                let count = abandon_commit_ids.len();
                let short_keep = &keep_commit_id[..keep_commit_id.len().min(8)];
                self.0.success(format!(
                    "Divergence resolved: kept {short_keep}, abandoned {count} version{}",
                    if count == 1 { "" } else { "s" }
                ));
            }
            Err(error_value) => {
                let details = format!("{error_value}");
                self.0.set_status(
                    error::set_error_with_details("Resolve divergence failed", &details),
                    MessageKind::Error,
                );
            }
        }
    }

    pub(super) fn run_rebase(
        &mut self,
        source: &str,
        dest: &str,
        rebase_type: RebaseType,
        allow_branches: bool,
    ) {
        let (text, kind) = operations::run_rebase(source, dest, rebase_type, allow_branches);
        self.0.set_status(text, kind);
    }

    pub(super) fn run_rebase_onto_trunk(&mut self, source: &str, rebase_type: RebaseType) {
        let (text, kind) = operations::run_rebase_onto_trunk(source, rebase_type);
        self.0.set_status(text, kind);
    }

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
