use super::super::super::{ModeState, ReduceCtx};

pub(super) fn push_select_toggle(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode {
        let filtered = state.filtered_bookmarks();
        if let Some(&(original_idx, _)) = filtered.get(state.cursor_index) {
            if state.selected.contains(&original_idx) {
                state.selected.remove(&original_idx);
            } else {
                state.selected.insert(original_idx);
            }
        }
    }
}

pub(super) fn push_select_all(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode {
        let filtered_indices: Vec<usize> = state
            .filtered_bookmarks()
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
        for idx in filtered_indices {
            state.selected.insert(idx);
        }
    }
}

pub(super) fn push_select_none(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode {
        let filtered_indices: Vec<usize> = state
            .filtered_bookmarks()
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();
        for idx in filtered_indices {
            state.selected.remove(&idx);
        }
    }
}
