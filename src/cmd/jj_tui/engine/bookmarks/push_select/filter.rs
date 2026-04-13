use super::super::super::{ModeState, ReduceCtx};
use super::super::helpers::previous_char_boundary;

pub(super) fn push_select_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode
        && state.cursor_index > 0
    {
        state.cursor_index -= 1;
    }
}

pub(super) fn push_select_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode {
        let filtered_count = state.filtered_bookmarks().len();
        if state.cursor_index < filtered_count.saturating_sub(1) {
            state.cursor_index += 1;
        }
    }
}

pub(super) fn push_select_filter_char(ctx: &mut ReduceCtx<'_>, ch: char) {
    if let ModeState::PushSelect(state) = ctx.mode {
        state.filter.insert(state.filter_cursor, ch);
        state.filter_cursor += ch.len_utf8();
        state.cursor_index = 0;
    }
}

pub(super) fn push_select_filter_backspace(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode
        && state.filter_cursor > 0
    {
        let prev = previous_char_boundary(&state.filter, state.filter_cursor);
        state.filter.remove(prev);
        state.filter_cursor = prev;
        state.cursor_index = 0;
    }
}
