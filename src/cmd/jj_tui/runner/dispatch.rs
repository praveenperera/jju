use super::super::effect::Effect;
use super::{RunCtx, bookmarks, git, interactive, revision};
use ratatui::DefaultTerminal;

pub(super) fn run_effect(ctx: &mut RunCtx<'_>, effect: Effect, terminal: &mut DefaultTerminal) {
    match effect {
        Effect::RefreshTree => ctx.refresh_tree(),
        Effect::SaveOperationForUndo => save_operation_for_undo(ctx),
        Effect::RunEdit { .. }
        | Effect::RunNew { .. }
        | Effect::RunCommit { .. }
        | Effect::RunAbandon { .. }
        | Effect::RunRebase { .. }
        | Effect::RunRebaseOntoTrunk { .. }
        | Effect::RunUndo
        | Effect::RunResolveDivergence { .. } => revision::handle(ctx, effect),
        Effect::RunGitPush { .. }
        | Effect::RunGitPushMultiple { .. }
        | Effect::RunGitPushAll
        | Effect::RunStackSync
        | Effect::RunGitFetch
        | Effect::RunGitImport
        | Effect::RunGitExport
        | Effect::RunCreatePR { .. } => git::handle(ctx, effect),
        Effect::RunBookmarkSet { .. }
        | Effect::RunBookmarkSetBackwards { .. }
        | Effect::RunBookmarkDelete { .. } => bookmarks::handle(ctx, effect),
        Effect::RunInteractive(operation) => interactive::handle(ctx, terminal, operation),
        Effect::SetStatus { text, kind } => ctx.set_status(text, kind),
        Effect::LoadConflictFiles => {}
    }
}

fn save_operation_for_undo(ctx: &mut RunCtx<'_>) {
    if let Ok(op_id) = crate::cmd::jj_tui::commands::get_current_op_id() {
        *ctx.last_op = Some(op_id);
    }
}
