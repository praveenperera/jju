mod confirm;
mod entry;
mod input;

use super::super::super::{MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::BookmarkSelectAction;

pub(super) fn enter_bookmark_picker(ctx: &mut ReduceCtx<'_>, action: BookmarkSelectAction) {
    entry::enter_bookmark_picker(ctx, action)
}

pub(super) fn bookmark_picker_up(ctx: &mut ReduceCtx<'_>) {
    input::bookmark_picker_up(ctx)
}

pub(super) fn bookmark_picker_down(ctx: &mut ReduceCtx<'_>) {
    input::bookmark_picker_down(ctx)
}

pub(super) fn bookmark_filter_char(ctx: &mut ReduceCtx<'_>, ch: char) {
    input::bookmark_filter_char(ctx, ch)
}

pub(super) fn bookmark_filter_backspace(ctx: &mut ReduceCtx<'_>) {
    input::bookmark_filter_backspace(ctx)
}

pub(super) fn confirm_bookmark_picker(ctx: &mut ReduceCtx<'_>) {
    if !matches!(ctx.mode, ModeState::BookmarkPicker(_)) {
        *ctx.mode = ModeState::Normal;
        return;
    }

    if !confirm::confirm_bookmark_picker(ctx) {
        ctx.set_status("No bookmark selected", MessageKind::Warning);
        *ctx.mode = ModeState::Normal;
    }
}
