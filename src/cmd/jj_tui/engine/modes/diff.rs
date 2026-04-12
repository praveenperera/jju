use super::super::selection::current_rev;
use super::super::{MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::handlers;
use crate::cmd::jj_tui::state::DiffState;

pub(super) fn enter_diff_view(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    if let Ok(diff_output) = super::super::super::commands::diff::get_diff(&rev) {
        let lines = handlers::diff::parse_diff(&diff_output, ctx.syntax_set, ctx.theme_set);
        *ctx.mode = ModeState::ViewingDiff(DiffState {
            lines,
            scroll_offset: 0,
            rev,
        });
    }
}

pub(super) fn scroll_up(ctx: &mut ReduceCtx<'_>, amount: usize) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = state.scroll_offset.saturating_sub(amount);
    }
}

pub(super) fn scroll_down(ctx: &mut ReduceCtx<'_>, amount: usize) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = state.scroll_offset.saturating_add(amount);
    }
}

pub(super) fn scroll_top(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = 0;
    }
}

pub(super) fn scroll_bottom(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = state.lines.len().saturating_sub(1);
    }
}
