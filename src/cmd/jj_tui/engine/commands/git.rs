use super::super::{Effect, ReduceCtx};

pub(super) fn run_simple_refresh(ctx: &mut ReduceCtx<'_>, effect: Effect) {
    ctx.effects.push(effect);
    ctx.effects.push(Effect::RefreshTree);
}
