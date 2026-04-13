use super::super::super::{Effect, MessageKind, ModeState, ReduceCtx};

pub(super) fn push_select_confirm(ctx: &mut ReduceCtx<'_>) {
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
