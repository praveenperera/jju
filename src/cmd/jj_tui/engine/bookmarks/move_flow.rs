use super::super::selection::get_rev_at_cursor;
use super::super::{Action, Effect, MessageKind, ModeState, ReduceCtx};
use super::{build_move_bookmark_picker_list, is_bookmark_move_backwards};
use crate::cmd::jj_tui::state::{
    BookmarkPickerState, BookmarkSelectAction, ConfirmAction, ConfirmState,
};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EnterMoveBookmarkMode => enter_move_bookmark_mode(ctx),
        Action::MoveBookmarkDestUp => move_bookmark_dest_up(ctx),
        Action::MoveBookmarkDestDown => move_bookmark_dest_down(ctx),
        Action::ExecuteBookmarkMove => execute_bookmark_move(ctx),
        _ => unreachable!("unsupported move bookmark action: {action:?}"),
    }
}

fn enter_move_bookmark_mode(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    let target_rev = node.change_id.clone();

    if let Ok(jj_repo) = crate::jj_lib_helpers::JjRepo::load(None) {
        let all_bookmarks = jj_repo.all_local_bookmarks();
        let pinned = node.bookmark_names();
        let all_bookmarks = build_move_bookmark_picker_list(all_bookmarks, pinned, ctx.tree);

        *ctx.mode = ModeState::BookmarkPicker(BookmarkPickerState {
            all_bookmarks,
            filter: String::new(),
            filter_cursor: 0,
            selected_index: 0,
            target_rev,
            action: BookmarkSelectAction::Move,
        });
    }
}

fn move_bookmark_dest_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::MovingBookmark(state) = ctx.mode
        && state.dest_cursor > 0
    {
        state.dest_cursor -= 1;
    }
}

fn move_bookmark_dest_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::MovingBookmark(state) = ctx.mode {
        let max = ctx.tree.visible_count().saturating_sub(1);
        if state.dest_cursor < max {
            state.dest_cursor += 1;
        }
    }
}

fn execute_bookmark_move(ctx: &mut ReduceCtx<'_>) {
    let ModeState::MovingBookmark(state) = &*ctx.mode else {
        *ctx.mode = ModeState::Normal;
        return;
    };

    let Some(dest) = get_rev_at_cursor(ctx.tree, state.dest_cursor) else {
        ctx.set_status("Invalid destination", MessageKind::Error);
        return;
    };

    let name = state.bookmark_name.clone();
    if is_bookmark_move_backwards(ctx.tree, &name, &dest) {
        let short_dest = &dest[..8.min(dest.len())];
        *ctx.mode = ModeState::Confirming(ConfirmState {
            action: ConfirmAction::MoveBookmarkBackwards {
                bookmark_name: name.clone(),
                dest_rev: dest.clone(),
            },
            message: format!(
                "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                name, short_dest
            ),
            revs: vec![],
        });
        return;
    }

    ctx.effects.push(Effect::RunBookmarkSet { name, rev: dest });
    ctx.effects.push(Effect::RefreshTree);
    *ctx.mode = ModeState::Normal;
}
