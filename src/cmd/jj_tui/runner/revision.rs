use super::{Effect, RunCtx, error, operations};
use crate::cmd::jj_tui::state::MessageKind;

pub(super) fn handle(ctx: &mut RunCtx<'_>, effect: Effect) {
    match effect {
        Effect::RunEdit { rev } => match crate::cmd::jj_tui::commands::revision::edit(&rev) {
            Ok(_) => ctx.success(format!("Now editing {rev}")),
            Err(error) => ctx.error(format!("Edit failed: {error}")),
        },
        Effect::RunNew { rev } => match crate::cmd::jj_tui::commands::revision::new(&rev) {
            Ok(_) => ctx.success("Created new commit"),
            Err(error) => ctx.error(format!("Failed: {error}")),
        },
        Effect::RunCommit { message } => {
            match crate::cmd::jj_tui::commands::revision::commit(&message) {
                Ok(_) => ctx.success("Changes committed"),
                Err(error) => ctx.error(format!("Commit failed: {error}")),
            }
        }
        Effect::RunAbandon { revset } => {
            match crate::cmd::jj_tui::commands::revision::abandon(&revset) {
                Ok(_) => {
                    let count = revset.matches('|').count() + 1;
                    if count == 1 {
                        ctx.success("Revision abandoned");
                    } else {
                        ctx.success(format!("{count} revisions abandoned"));
                    }
                }
                Err(error) => {
                    let details = format!("{error}");
                    ctx.set_status(
                        error::set_error_with_details("Abandon failed", &details),
                        MessageKind::Error,
                    );
                }
            }
        }
        Effect::RunRebase {
            source,
            dest,
            rebase_type,
            allow_branches,
        } => {
            let (text, kind) = operations::run_rebase(&source, &dest, rebase_type, allow_branches);
            ctx.set_status(text, kind);
        }
        Effect::RunRebaseOntoTrunk {
            source,
            rebase_type,
        } => {
            let (text, kind) = operations::run_rebase_onto_trunk(&source, rebase_type);
            ctx.set_status(text, kind);
        }
        Effect::RunUndo => match ctx.last_op.take() {
            Some(op_id) if !op_id.is_empty() => {
                match crate::cmd::jj_tui::commands::restore_op(&op_id) {
                    Ok(_) => ctx.success("Operation undone"),
                    Err(error) => ctx.error(format!("Undo failed: {error}")),
                }
            }
            _ => ctx.warn("Nothing to undo"),
        },
        Effect::RunResolveDivergence {
            keep_commit_id,
            abandon_commit_ids,
        } => run_resolve_divergence(ctx, &keep_commit_id, abandon_commit_ids),
        _ => unreachable!("unsupported revision effect: {effect:?}"),
    }
}

fn run_resolve_divergence(
    ctx: &mut RunCtx<'_>,
    keep_commit_id: &str,
    abandon_commit_ids: Vec<String>,
) {
    let revset = abandon_commit_ids.join(" | ");
    match crate::cmd::jj_tui::commands::revision::abandon(&revset) {
        Ok(_) => {
            let count = abandon_commit_ids.len();
            let short_keep = &keep_commit_id[..keep_commit_id.len().min(8)];
            ctx.success(format!(
                "Divergence resolved: kept {short_keep}, abandoned {count} version{}",
                if count == 1 { "" } else { "s" }
            ));
        }
        Err(error) => {
            let details = format!("{error}");
            ctx.set_status(
                error::set_error_with_details("Resolve divergence failed", &details),
                MessageKind::Error,
            );
        }
    }
}
