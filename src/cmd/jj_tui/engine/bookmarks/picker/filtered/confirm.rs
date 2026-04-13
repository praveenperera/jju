use super::super::super::{Effect, ModeState, ReduceCtx};
use crate::cmd::jj_tui::engine::bookmarks::helpers::{
    bookmark_is_on_rev, is_bookmark_move_backwards,
};
use crate::cmd::jj_tui::state::{
    BookmarkSelectAction, ConfirmAction, ConfirmState, MovingBookmarkState,
};

pub(super) fn confirm_bookmark_picker(ctx: &mut ReduceCtx<'_>) -> bool {
    let Some(state) = bookmark_picker_state(ctx) else {
        return false;
    };

    let selected_bookmark = state
        .filtered_bookmarks()
        .get(state.selected_index)
        .map(|bookmark| (*bookmark).clone());
    let target_rev = state.target_rev.clone();
    let action = state.action;
    let filter = state.filter.trim().to_string();

    let Some(bookmark_name) = selected_bookmark else {
        return create_new_bookmark(ctx, action, &filter, &target_rev);
    };

    match action {
        BookmarkSelectAction::Move => {
            confirm_move_bookmark_picker(ctx, bookmark_name, target_rev);
        }
        BookmarkSelectAction::Delete => {
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects.push(Effect::RunBookmarkDelete {
                name: bookmark_name,
            });
            ctx.effects.push(Effect::RefreshTree);
            *ctx.mode = ModeState::Normal;
        }
        BookmarkSelectAction::CreatePR => {
            ctx.effects.push(Effect::RunCreatePR {
                bookmark: bookmark_name,
            });
            *ctx.mode = ModeState::Normal;
        }
    }

    true
}

fn bookmark_picker_state<'a>(
    ctx: &'a ReduceCtx<'_>,
) -> Option<&'a crate::cmd::jj_tui::state::BookmarkPickerState> {
    match &*ctx.mode {
        ModeState::BookmarkPicker(state) => Some(state),
        ModeState::Normal
        | ModeState::Selecting
        | ModeState::Rebasing(_)
        | ModeState::Squashing(_)
        | ModeState::ViewingDiff(_)
        | ModeState::Confirming(_)
        | ModeState::MovingBookmark(_)
        | ModeState::BookmarkSelect(_)
        | ModeState::PushSelect(_)
        | ModeState::Help(_)
        | ModeState::Conflicts(_) => None,
    }
}

fn create_new_bookmark(
    ctx: &mut ReduceCtx<'_>,
    action: BookmarkSelectAction,
    name: &str,
    target_rev: &str,
) -> bool {
    if action != BookmarkSelectAction::Move || name.is_empty() {
        return false;
    }

    ctx.effects.push(Effect::SaveOperationForUndo);
    ctx.effects.push(Effect::RunBookmarkSet {
        name: name.to_string(),
        rev: target_rev.to_string(),
    });
    ctx.effects.push(Effect::RefreshTree);
    *ctx.mode = ModeState::Normal;
    true
}

fn confirm_move_bookmark_picker(
    ctx: &mut ReduceCtx<'_>,
    bookmark_name: String,
    target_rev: String,
) {
    if bookmark_is_on_rev(ctx.tree, &bookmark_name, &target_rev) {
        *ctx.mode = ModeState::MovingBookmark(MovingBookmarkState {
            bookmark_name,
            dest_cursor: ctx.tree.view.cursor,
        });
        ctx.effects.push(Effect::SaveOperationForUndo);
        return;
    }

    if is_bookmark_move_backwards(ctx.tree, &bookmark_name, &target_rev) {
        let short_dest = &target_rev[..8.min(target_rev.len())];
        *ctx.mode = ModeState::Confirming(ConfirmState {
            action: ConfirmAction::MoveBookmarkBackwards {
                bookmark_name: bookmark_name.clone(),
                dest_rev: target_rev.clone(),
            },
            message: format!(
                "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                bookmark_name, short_dest
            ),
            revs: vec![],
        });
        ctx.effects.push(Effect::SaveOperationForUndo);
        return;
    }

    ctx.effects.push(Effect::SaveOperationForUndo);
    ctx.effects.push(Effect::RunBookmarkSet {
        name: bookmark_name,
        rev: target_rev,
    });
    ctx.effects.push(Effect::RefreshTree);
    *ctx.mode = ModeState::Normal;
}
