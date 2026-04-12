use super::super::rebase::compute_moving_indices;
use super::super::selection::{current_rev, get_rev_at_cursor};
use super::super::{Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::RebaseState;
use crate::cmd::jj_tui::state::RebaseType;

pub(super) fn enter(ctx: &mut ReduceCtx<'_>, rebase_type: RebaseType) {
    let source_rev = current_rev(ctx.tree);
    if source_rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let current = ctx.tree.view.cursor;
    let state = RebaseState {
        source_rev: source_rev.clone(),
        rebase_type,
        dest_cursor: current,
        allow_branches: false,
    };

    *ctx.mode = ModeState::Rebasing(state);

    let moving = compute_moving_indices(ctx.tree, ctx.mode);
    let max = ctx.tree.visible_count();
    let source_struct_depth = ctx
        .tree
        .visible_entries()
        .get(current)
        .map(|entry| ctx.tree.nodes()[entry.node_index].depth)
        .unwrap_or(0);

    let mut initial_cursor = current.saturating_sub(1);
    while initial_cursor > 0 {
        let entry = &ctx.tree.visible_entries()[initial_cursor];
        let node = &ctx.tree.nodes()[entry.node_index];
        if node.depth < source_struct_depth && !moving.contains(&initial_cursor) {
            break;
        }
        initial_cursor -= 1;
    }

    if moving.contains(&initial_cursor) || initial_cursor >= max {
        initial_cursor = 0;
        while initial_cursor < max && moving.contains(&initial_cursor) {
            initial_cursor += 1;
        }
    }

    if let ModeState::Rebasing(state) = ctx.mode {
        state.dest_cursor = initial_cursor;
    }

    ctx.effects.push(Effect::SaveOperationForUndo);
}

pub(super) fn move_dest_up(ctx: &mut ReduceCtx<'_>) {
    let moving = compute_moving_indices(ctx.tree, ctx.mode);
    if let ModeState::Rebasing(state) = ctx.mode {
        let mut next = state.dest_cursor.saturating_sub(1);
        while next > 0 && moving.contains(&next) {
            next -= 1;
        }
        if !moving.contains(&next) {
            state.dest_cursor = next;
        }
    }
}

pub(super) fn move_dest_down(ctx: &mut ReduceCtx<'_>) {
    let moving = compute_moving_indices(ctx.tree, ctx.mode);
    let max = ctx.tree.visible_count();
    if let ModeState::Rebasing(state) = ctx.mode {
        let mut next = state.dest_cursor + 1;
        while next < max && moving.contains(&next) {
            next += 1;
        }
        if next < max {
            state.dest_cursor = next;
        }
    }
}

pub(super) fn toggle_branches(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Rebasing(state) = ctx.mode {
        state.allow_branches = !state.allow_branches;
    }
}

pub(super) fn execute(ctx: &mut ReduceCtx<'_>) {
    let ModeState::Rebasing(state) = &*ctx.mode else {
        *ctx.mode = ModeState::Normal;
        return;
    };

    let Some(dest) = get_rev_at_cursor(ctx.tree, state.dest_cursor) else {
        ctx.set_status("Invalid destination", MessageKind::Error);
        return;
    };

    if state.source_rev == dest {
        ctx.set_status("Cannot rebase onto self", MessageKind::Error);
        return;
    }

    ctx.effects.push(Effect::RunRebase {
        source: state.source_rev.clone(),
        dest,
        rebase_type: state.rebase_type,
        allow_branches: state.allow_branches,
    });
    ctx.effects.push(Effect::RefreshTree);
    *ctx.mode = ModeState::Normal;
}
