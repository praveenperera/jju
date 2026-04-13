mod filtered;
mod select;

use super::super::{Action, ReduceCtx};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EnterBookmarkPicker(action) => filtered::enter_bookmark_picker(ctx, action),
        Action::SelectBookmarkUp => select::select_bookmark_up(ctx),
        Action::SelectBookmarkDown => select::select_bookmark_down(ctx),
        Action::ConfirmBookmarkSelect => select::confirm_bookmark_select(ctx),
        Action::BookmarkPickerUp => filtered::bookmark_picker_up(ctx),
        Action::BookmarkPickerDown => filtered::bookmark_picker_down(ctx),
        Action::BookmarkFilterChar(ch) => filtered::bookmark_filter_char(ctx, ch),
        Action::BookmarkFilterBackspace => filtered::bookmark_filter_backspace(ctx),
        Action::ConfirmBookmarkPicker => filtered::confirm_bookmark_picker(ctx),
        _ => unreachable!("unsupported bookmark picker action: {action:?}"),
    }
}
