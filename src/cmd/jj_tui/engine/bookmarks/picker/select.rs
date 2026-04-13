use super::super::super::{Effect, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::{BookmarkSelectAction, MovingBookmarkState};

pub(super) fn select_bookmark_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkSelect(state) = ctx.mode
        && state.selected_index > 0
    {
        state.selected_index -= 1;
    }
}

pub(super) fn select_bookmark_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkSelect(state) = ctx.mode
        && state.selected_index < state.bookmarks.len().saturating_sub(1)
    {
        state.selected_index += 1;
    }
}

pub(super) fn confirm_bookmark_select(ctx: &mut ReduceCtx<'_>) {
    let ModeState::BookmarkSelect(state) = &*ctx.mode else {
        *ctx.mode = ModeState::Normal;
        return;
    };

    let bookmark = state.bookmarks[state.selected_index].clone();
    match state.action {
        BookmarkSelectAction::Move => {
            *ctx.mode = ModeState::MovingBookmark(MovingBookmarkState {
                bookmark_name: bookmark,
                dest_cursor: ctx.tree.view.cursor,
            });
            ctx.effects.push(Effect::SaveOperationForUndo);
        }
        BookmarkSelectAction::Delete => {
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects
                .push(Effect::RunBookmarkDelete { name: bookmark });
            ctx.effects.push(Effect::RefreshTree);
            *ctx.mode = ModeState::Normal;
        }
        BookmarkSelectAction::CreatePR => {
            ctx.effects.push(Effect::RunCreatePR { bookmark });
            *ctx.mode = ModeState::Normal;
        }
    }
}
