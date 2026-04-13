use super::super::super::{ModeState, ReduceCtx};
use crate::cmd::jj_tui::engine::bookmarks::helpers::previous_char_boundary;

pub(super) fn bookmark_picker_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkPicker(state) = ctx.mode
        && state.selected_index > 0
    {
        state.selected_index -= 1;
    }
}

pub(super) fn bookmark_picker_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkPicker(state) = ctx.mode {
        let filtered_count = state.filtered_bookmarks().len();
        if state.selected_index < filtered_count.saturating_sub(1) {
            state.selected_index += 1;
        }
    }
}

pub(super) fn bookmark_filter_char(ctx: &mut ReduceCtx<'_>, ch: char) {
    if let ModeState::BookmarkPicker(state) = ctx.mode {
        state.filter.insert(state.filter_cursor, ch);
        state.filter_cursor += ch.len_utf8();
        state.selected_index = 0;
    }
}

pub(super) fn bookmark_filter_backspace(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkPicker(state) = ctx.mode
        && state.filter_cursor > 0
    {
        let prev = previous_char_boundary(&state.filter, state.filter_cursor);
        state.filter.remove(prev);
        state.filter_cursor = prev;
        state.selected_index = 0;
    }
}
