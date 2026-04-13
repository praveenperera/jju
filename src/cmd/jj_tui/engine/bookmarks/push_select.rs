mod confirm;
mod filter;
mod mode;
mod selection;

use super::super::{Action, ReduceCtx};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::GitPush => mode::git_push(ctx),
        Action::PushSelectUp => filter::push_select_up(ctx),
        Action::PushSelectDown => filter::push_select_down(ctx),
        Action::PushSelectFilterChar(ch) => filter::push_select_filter_char(ctx, ch),
        Action::PushSelectFilterBackspace => filter::push_select_filter_backspace(ctx),
        Action::PushSelectToggle => selection::push_select_toggle(ctx),
        Action::PushSelectAll => selection::push_select_all(ctx),
        Action::PushSelectNone => selection::push_select_none(ctx),
        Action::PushSelectConfirm => confirm::push_select_confirm(ctx),
        _ => unreachable!("unsupported push select action: {action:?}"),
    }
}
