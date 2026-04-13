use super::{Action, ReduceCtx};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::Quit => *ctx.should_quit = true,
        Action::Noop => {}
        Action::SetPendingKey(prefix) => *ctx.pending_key = Some(prefix),
        Action::ClearPendingKey => *ctx.pending_key = None,
        _ => unreachable!("unsupported lifecycle action: {action:?}"),
    }
}
