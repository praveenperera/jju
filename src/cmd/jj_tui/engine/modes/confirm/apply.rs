use super::super::super::selection::current_rev;
use super::super::{Effect, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::ConfirmAction;

pub(super) fn confirm_yes(ctx: &mut ReduceCtx<'_>) {
    let ModeState::Confirming(state) = std::mem::replace(ctx.mode, ModeState::Normal) else {
        return;
    };

    match state.action {
        ConfirmAction::Abandon => abandon(ctx, state.revs),
        ConfirmAction::StackSync => stack_sync(ctx),
        ConfirmAction::RebaseOntoTrunk(rebase_type) => rebase_onto_trunk(ctx, rebase_type),
        ConfirmAction::MoveBookmarkBackwards {
            bookmark_name,
            dest_rev,
        } => move_bookmark_backwards(ctx, bookmark_name, dest_rev),
    }

    ctx.tree.clear_selection();
}

fn abandon(ctx: &mut ReduceCtx<'_>, revs: Vec<String>) {
    let revset = revs.join(" | ");
    push_with_refresh(ctx, Effect::RunAbandon { revset });
}

fn stack_sync(ctx: &mut ReduceCtx<'_>) {
    push_with_refresh(ctx, Effect::RunStackSync);
}

fn rebase_onto_trunk(ctx: &mut ReduceCtx<'_>, rebase_type: crate::cmd::jj_tui::state::RebaseType) {
    let source = current_rev(ctx.tree);
    if source.is_empty() {
        ctx.set_status(
            "No revision selected",
            crate::cmd::jj_tui::state::MessageKind::Error,
        );
        return;
    }

    push_with_refresh(
        ctx,
        Effect::RunRebaseOntoTrunk {
            source,
            rebase_type,
        },
    );
}

fn move_bookmark_backwards(ctx: &mut ReduceCtx<'_>, bookmark_name: String, dest_rev: String) {
    push_with_refresh(
        ctx,
        Effect::RunBookmarkSetBackwards {
            name: bookmark_name,
            rev: dest_rev,
        },
    );
}

fn push_with_refresh(ctx: &mut ReduceCtx<'_>, effect: Effect) {
    ctx.effects.push(Effect::SaveOperationForUndo);
    ctx.effects.push(effect);
    ctx.effects.push(Effect::RefreshTree);
}
