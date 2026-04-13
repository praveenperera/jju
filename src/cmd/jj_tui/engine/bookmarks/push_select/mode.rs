use super::super::super::{Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::PushSelectState;
use ahash::HashSet;

pub(super) fn git_push(ctx: &mut ReduceCtx<'_>) {
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
