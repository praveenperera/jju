mod apply;
mod enter;

use super::super::ReduceCtx;
use crate::cmd::jj_tui::state::RebaseType;

pub(super) fn enter_stack_sync(ctx: &mut ReduceCtx<'_>) {
    enter::stack_sync(ctx);
}

pub(super) fn enter_abandon(ctx: &mut ReduceCtx<'_>) {
    enter::abandon(ctx);
}

pub(super) fn enter_rebase_onto_trunk(ctx: &mut ReduceCtx<'_>, rebase_type: RebaseType) {
    enter::rebase_onto_trunk(ctx, rebase_type);
}

pub(super) fn confirm_yes(ctx: &mut ReduceCtx<'_>) {
    apply::confirm_yes(ctx);
}
