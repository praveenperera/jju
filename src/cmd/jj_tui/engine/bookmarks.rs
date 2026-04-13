mod helpers;
mod move_flow;
mod picker;
mod push_select;

use super::{Action, Effect, ModeState, ReduceCtx};

#[cfg(test)]
use crate::cmd::jj_tui::tree::TreeState;

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EnterMoveBookmarkMode
        | Action::MoveBookmarkDestUp
        | Action::MoveBookmarkDestDown
        | Action::ExecuteBookmarkMove => move_flow::handle(ctx, action),
        Action::EnterBookmarkPicker(_)
        | Action::SelectBookmarkUp
        | Action::SelectBookmarkDown
        | Action::ConfirmBookmarkSelect
        | Action::BookmarkPickerUp
        | Action::BookmarkPickerDown
        | Action::BookmarkFilterChar(_)
        | Action::BookmarkFilterBackspace
        | Action::ConfirmBookmarkPicker => picker::handle(ctx, action),
        Action::GitPush
        | Action::PushSelectUp
        | Action::PushSelectDown
        | Action::PushSelectToggle
        | Action::PushSelectAll
        | Action::PushSelectNone
        | Action::PushSelectFilterChar(_)
        | Action::PushSelectFilterBackspace
        | Action::PushSelectConfirm => push_select::handle(ctx, action),
        Action::GitPushAll => {
            ctx.effects.push(Effect::RunGitPushAll);
            ctx.effects.push(Effect::RefreshTree);
        }
        Action::ExitBookmarkMode | Action::ExitPushSelect => *ctx.mode = ModeState::Normal,
        _ => unreachable!("unsupported bookmark action: {action:?}"),
    }
}

#[cfg(test)]
pub(super) fn build_move_bookmark_picker_list(
    all_bookmarks: Vec<String>,
    pinned: Vec<String>,
    tree: &TreeState,
) -> Vec<String> {
    helpers::build_move_bookmark_picker_list(all_bookmarks, pinned, tree)
}
