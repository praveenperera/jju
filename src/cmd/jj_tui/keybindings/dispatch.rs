use super::mode_id_from_state;
use super::{Action, ModeId};
use crate::cmd::jj_tui::controller::ControllerContext;
use ratatui::crossterm::event::KeyEvent;

pub(crate) fn dispatch_key(ctx: &ControllerContext<'_>, key: KeyEvent) -> Action {
    let mode = mode_id_from_state(ctx.mode);

    if let Some(pending) = ctx.pending_key {
        return match_binding(ctx, mode, Some(pending), &key).unwrap_or(Action::ClearPendingKey);
    }

    match_binding(ctx, mode, None, &key).unwrap_or(Action::Noop)
}

fn match_binding(
    ctx: &ControllerContext<'_>,
    mode: ModeId,
    pending_prefix: Option<char>,
    key: &KeyEvent,
) -> Option<Action> {
    for binding in super::bindings() {
        if binding.mode != mode || binding.pending_prefix != pending_prefix {
            continue;
        }
        if let Some(captured) = binding.key.matches(key) {
            return Some(binding.action.build(ctx, captured.char()));
        }
    }
    None
}
