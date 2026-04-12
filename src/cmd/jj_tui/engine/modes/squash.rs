use super::super::selection::{current_rev, get_rev_at_cursor};
use super::super::{Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::SquashState;
use jju_core::interactive::{InteractiveOperation, SquashOperation};

pub(super) fn enter(ctx: &mut ReduceCtx<'_>) {
    let source_rev = current_rev(ctx.tree);
    if source_rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let current = ctx.tree.view.cursor;
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
        if node.depth < source_struct_depth {
            break;
        }
        initial_cursor -= 1;
    }

    *ctx.mode = ModeState::Squashing(SquashState {
        source_rev,
        dest_cursor: initial_cursor,
        op_before: String::new(),
    });
    ctx.effects.push(Effect::SaveOperationForUndo);
}

pub(super) fn move_dest_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Squashing(state) = ctx.mode
        && state.dest_cursor > 0
    {
        state.dest_cursor -= 1;
    }
}

pub(super) fn move_dest_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Squashing(state) = ctx.mode {
        let max = ctx.tree.visible_count().saturating_sub(1);
        if state.dest_cursor < max {
            state.dest_cursor += 1;
        }
    }
}

pub(super) fn execute(ctx: &mut ReduceCtx<'_>) {
    let ModeState::Squashing(state) = &*ctx.mode else {
        *ctx.mode = ModeState::Normal;
        return;
    };

    let Some(target) = get_rev_at_cursor(ctx.tree, state.dest_cursor) else {
        ctx.set_status("Invalid target", MessageKind::Error);
        return;
    };

    if state.source_rev == target {
        ctx.set_status("Cannot squash into self", MessageKind::Error);
        return;
    }

    ctx.effects
        .push(Effect::RunInteractive(InteractiveOperation::Squash(
            SquashOperation {
                source_rev: state.source_rev.clone(),
                target_rev: target,
                op_before: state.op_before.clone(),
            },
        )));
    *ctx.mode = ModeState::Normal;
}
