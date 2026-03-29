use super::super::{Action, Effect, MessageKind, ModeState, ReduceCtx};
use super::previous_char_boundary;
use crate::cmd::jj_tui::state::PushSelectState;
use ahash::HashSet;

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::GitPush => git_push(ctx),
        Action::PushSelectUp => push_select_up(ctx),
        Action::PushSelectDown => push_select_down(ctx),
        Action::PushSelectToggle => push_select_toggle(ctx),
        Action::PushSelectAll => push_select_all(ctx),
        Action::PushSelectNone => push_select_none(ctx),
        Action::PushSelectFilterChar(ch) => push_select_filter_char(ctx, ch),
        Action::PushSelectFilterBackspace => push_select_filter_backspace(ctx),
        Action::PushSelectConfirm => push_select_confirm(ctx),
        _ => unreachable!("unsupported push select action: {action:?}"),
    }
}

fn git_push(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if node.bookmarks.is_empty() {
        ctx.set_status("No bookmark on this revision to push", MessageKind::Warning);
        return;
    }

    if node.bookmarks.len() == 1 {
        let bookmark = node.bookmarks[0].name.clone();
        ctx.effects.push(Effect::RunGitPush { bookmark });
        ctx.effects.push(Effect::RefreshTree);
        return;
    }

    let all_bookmarks: Vec<String> = node
        .bookmarks
        .iter()
        .map(|bookmark| bookmark.name.clone())
        .collect();
    let selected: HashSet<usize> = (0..all_bookmarks.len()).collect();
    *ctx.mode = ModeState::PushSelect(PushSelectState {
        all_bookmarks,
        filter: String::new(),
        filter_cursor: 0,
        cursor_index: 0,
        selected,
    });
}

fn push_select_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode
        && state.cursor_index > 0
    {
        state.cursor_index -= 1;
    }
}

fn push_select_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode {
        let filtered_count = state.filtered_bookmarks().len();
        if state.cursor_index < filtered_count.saturating_sub(1) {
            state.cursor_index += 1;
        }
    }
}

fn push_select_toggle(ctx: &mut ReduceCtx<'_>) {
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

fn push_select_all(ctx: &mut ReduceCtx<'_>) {
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

fn push_select_none(ctx: &mut ReduceCtx<'_>) {
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

fn push_select_filter_char(ctx: &mut ReduceCtx<'_>, ch: char) {
    if let ModeState::PushSelect(state) = ctx.mode {
        state.filter.insert(state.filter_cursor, ch);
        state.filter_cursor += ch.len_utf8();
        state.cursor_index = 0;
    }
}

fn push_select_filter_backspace(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::PushSelect(state) = ctx.mode
        && state.filter_cursor > 0
    {
        let prev = previous_char_boundary(&state.filter, state.filter_cursor);
        state.filter.remove(prev);
        state.filter_cursor = prev;
        state.cursor_index = 0;
    }
}

fn push_select_confirm(ctx: &mut ReduceCtx<'_>) {
    let ModeState::PushSelect(state) = &*ctx.mode else {
        *ctx.mode = ModeState::Normal;
        return;
    };

    if state.selected.is_empty() {
        ctx.set_status("No bookmarks selected", MessageKind::Warning);
        *ctx.mode = ModeState::Normal;
        return;
    }

    let bookmarks: Vec<String> = state
        .selected
        .iter()
        .filter_map(|&idx| state.all_bookmarks.get(idx).cloned())
        .collect();

    ctx.effects.push(Effect::RunGitPushMultiple { bookmarks });
    ctx.effects.push(Effect::RefreshTree);
    *ctx.mode = ModeState::Normal;
}
