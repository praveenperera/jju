use super::super::{Action, Effect, MessageKind, ModeState, ReduceCtx};
use super::{bookmark_is_on_rev, is_bookmark_move_backwards, previous_char_boundary};
use crate::cmd::jj_tui::state::{
    BookmarkPickerState, BookmarkSelectAction, ConfirmAction, ConfirmState, MovingBookmarkState,
};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EnterBookmarkPicker(action) => enter_bookmark_picker(ctx, action),
        Action::SelectBookmarkUp => select_bookmark_up(ctx),
        Action::SelectBookmarkDown => select_bookmark_down(ctx),
        Action::ConfirmBookmarkSelect => confirm_bookmark_select(ctx),
        Action::BookmarkPickerUp => bookmark_picker_up(ctx),
        Action::BookmarkPickerDown => bookmark_picker_down(ctx),
        Action::BookmarkFilterChar(ch) => bookmark_filter_char(ctx, ch),
        Action::BookmarkFilterBackspace => bookmark_filter_backspace(ctx),
        Action::ConfirmBookmarkPicker => confirm_bookmark_picker(ctx),
        _ => unreachable!("unsupported bookmark picker action: {action:?}"),
    }
}

fn enter_bookmark_picker(ctx: &mut ReduceCtx<'_>, action: BookmarkSelectAction) {
    if let Ok(jj_repo) = crate::jj_lib_helpers::JjRepo::load(None) {
        let mut all_bookmarks = jj_repo.all_local_bookmarks();

        if all_bookmarks.is_empty() {
            ctx.set_status("No bookmarks in repository", MessageKind::Warning);
            return;
        }

        if action == BookmarkSelectAction::Delete {
            let current_bookmarks: Vec<String> = ctx
                .tree
                .current_node()
                .map(|node| node.bookmark_names())
                .unwrap_or_default();

            if !current_bookmarks.is_empty() {
                all_bookmarks.retain(|bookmark| !current_bookmarks.contains(bookmark));
                let mut reordered = current_bookmarks;
                reordered.extend(all_bookmarks);
                all_bookmarks = reordered;
            }
        }

        let target_rev = ctx
            .tree
            .current_node()
            .map(|node| node.change_id.clone())
            .unwrap_or_default();

        *ctx.mode = ModeState::BookmarkPicker(BookmarkPickerState {
            all_bookmarks,
            filter: String::new(),
            filter_cursor: 0,
            selected_index: 0,
            target_rev,
            action,
        });
    }
}

fn select_bookmark_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkSelect(state) = ctx.mode
        && state.selected_index > 0
    {
        state.selected_index -= 1;
    }
}

fn select_bookmark_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkSelect(state) = ctx.mode
        && state.selected_index < state.bookmarks.len().saturating_sub(1)
    {
        state.selected_index += 1;
    }
}

fn confirm_bookmark_select(ctx: &mut ReduceCtx<'_>) {
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

fn bookmark_picker_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkPicker(state) = ctx.mode
        && state.selected_index > 0
    {
        state.selected_index -= 1;
    }
}

fn bookmark_picker_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkPicker(state) = ctx.mode {
        let filtered_count = state.filtered_bookmarks().len();
        if state.selected_index < filtered_count.saturating_sub(1) {
            state.selected_index += 1;
        }
    }
}

fn bookmark_filter_char(ctx: &mut ReduceCtx<'_>, ch: char) {
    if let ModeState::BookmarkPicker(state) = ctx.mode {
        state.filter.insert(state.filter_cursor, ch);
        state.filter_cursor += ch.len_utf8();
        state.selected_index = 0;
    }
}

fn bookmark_filter_backspace(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::BookmarkPicker(state) = ctx.mode
        && state.filter_cursor > 0
    {
        let prev = previous_char_boundary(&state.filter, state.filter_cursor);
        state.filter.remove(prev);
        state.filter_cursor = prev;
        state.selected_index = 0;
    }
}

fn confirm_bookmark_picker(ctx: &mut ReduceCtx<'_>) {
    let ModeState::BookmarkPicker(state) = &*ctx.mode else {
        *ctx.mode = ModeState::Normal;
        return;
    };

    let filtered = state.filtered_bookmarks();
    let target_rev = state.target_rev.clone();
    let action = state.action;

    let Some(bookmark) = filtered.get(state.selected_index) else {
        if action == BookmarkSelectAction::Move && !state.filter.trim().is_empty() {
            let name = state.filter.trim().to_string();
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects.push(Effect::RunBookmarkSet {
                name,
                rev: target_rev,
            });
            ctx.effects.push(Effect::RefreshTree);
            *ctx.mode = ModeState::Normal;
            return;
        }

        ctx.set_status("No bookmark selected", MessageKind::Warning);
        *ctx.mode = ModeState::Normal;
        return;
    };

    let bookmark_name = (*bookmark).clone();
    match action {
        BookmarkSelectAction::Move => {
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
            } else {
                ctx.effects.push(Effect::SaveOperationForUndo);
                ctx.effects.push(Effect::RunBookmarkSet {
                    name: bookmark_name,
                    rev: target_rev,
                });
                ctx.effects.push(Effect::RefreshTree);
                *ctx.mode = ModeState::Normal;
            }
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
}
