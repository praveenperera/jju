use super::super::super::{ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::{BookmarkPickerState, BookmarkSelectAction, MessageKind};
use crate::jj_lib_helpers::JjRepo;

pub(super) fn enter_bookmark_picker(ctx: &mut ReduceCtx<'_>, action: BookmarkSelectAction) {
    let Ok(jj_repo) = JjRepo::load(None) else {
        return;
    };

    let mut all_bookmarks = jj_repo.all_local_bookmarks();
    if all_bookmarks.is_empty() {
        ctx.set_status("No bookmarks in repository", MessageKind::Warning);
        return;
    }

    if action == BookmarkSelectAction::Delete {
        reorder_delete_bookmarks(ctx, &mut all_bookmarks);
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

fn reorder_delete_bookmarks(ctx: &ReduceCtx<'_>, all_bookmarks: &mut Vec<String>) {
    let current_bookmarks: Vec<String> = ctx
        .tree
        .current_node()
        .map(|node| node.bookmark_names())
        .unwrap_or_default();

    if current_bookmarks.is_empty() {
        return;
    }

    all_bookmarks.retain(|bookmark| !current_bookmarks.contains(bookmark));
    let mut reordered = current_bookmarks;
    reordered.extend(all_bookmarks.iter().cloned());
    *all_bookmarks = reordered;
}
