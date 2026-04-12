use super::super::{Effect, ModeState, ReduceCtx};
use jju_core::interactive::InteractiveOperation;

pub(super) fn move_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Conflicts(state) = ctx.mode
        && state.selected_index > 0
    {
        state.selected_index -= 1;
    }
}

pub(super) fn move_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Conflicts(state) = ctx.mode {
        let max = state.files.len().saturating_sub(1);
        if state.selected_index < max {
            state.selected_index += 1;
        }
    }
}

pub(super) fn start_resolve(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Conflicts(state) = ctx.mode
        && let Some(file) = state.files.get(state.selected_index).cloned()
    {
        ctx.effects
            .push(Effect::RunInteractive(InteractiveOperation::Resolve {
                file,
            }));
        *ctx.mode = ModeState::Normal;
    }
}
