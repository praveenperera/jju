use super::super::{Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::{BookmarkSelectAction, BookmarkSelectState};

pub(super) fn create_pr(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if node.bookmarks.is_empty() {
        ctx.set_status(
            "No bookmark on this revision to create PR from",
            MessageKind::Warning,
        );
        return;
    }

    if node.bookmarks.len() == 1 {
        let bookmark = node.bookmarks[0].name.clone();
        ctx.effects.push(Effect::RunCreatePR { bookmark });
        return;
    }

    let bookmarks: Vec<String> = node
        .bookmarks
        .iter()
        .map(|bookmark| bookmark.name.clone())
        .collect();
    let target_rev = node.change_id.clone();
    *ctx.mode = ModeState::BookmarkSelect(BookmarkSelectState {
        bookmarks,
        selected_index: 0,
        target_rev,
        action: BookmarkSelectAction::CreatePR,
    });
}
