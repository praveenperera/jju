use super::super::selection::{current_rev, get_rev_at_cursor, selected_revs_in_visible_order};
use super::super::{Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::SquashState;
use jju_core::interactive::{InteractiveOperation, SquashOperation};

pub(super) fn enter(ctx: &mut ReduceCtx<'_>) {
    let source_revs = source_revs_for_squash(ctx);
    if source_revs.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let Some(dest_cursor) = initial_dest_cursor(ctx, &source_revs) else {
        ctx.set_status("No valid squash target", MessageKind::Error);
        return;
    };

    *ctx.mode = ModeState::Squashing(SquashState {
        source_revs,
        dest_cursor,
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

    let source_revs: Vec<String> = state
        .source_revs
        .iter()
        .filter(|rev| *rev != &target)
        .cloned()
        .collect();
    if source_revs.is_empty() {
        ctx.set_status("Select at least one source revision", MessageKind::Error);
        return;
    }

    ctx.effects
        .push(Effect::RunInteractive(InteractiveOperation::Squash(
            SquashOperation {
                source_revs,
                target_rev: target,
                op_before: state.op_before.clone(),
            },
        )));
    *ctx.mode = ModeState::Normal;
}

fn source_revs_for_squash(ctx: &ReduceCtx<'_>) -> Vec<String> {
    if ctx.tree.view.selected.is_empty() {
        let source_rev = current_rev(ctx.tree);
        if source_rev.is_empty() {
            Vec::new()
        } else {
            vec![source_rev]
        }
    } else {
        selected_revs_in_visible_order(ctx.tree)
    }
}

fn initial_dest_cursor(ctx: &ReduceCtx<'_>, source_revs: &[String]) -> Option<usize> {
    if source_revs.len() == 1 && ctx.tree.view.selected.is_empty() {
        return single_source_dest_cursor(ctx);
    }

    let selected_indices: Vec<usize> = ctx
        .tree
        .visible_entries()
        .iter()
        .enumerate()
        .filter(|(index, _entry)| ctx.tree.view.selected.contains(index))
        .map(|(index, _entry)| index)
        .collect();
    let top_selected = selected_indices.first().copied()?;

    (0..top_selected)
        .rev()
        .find(|index| !ctx.tree.view.selected.contains(index))
        .or_else(|| {
            ctx.tree
                .visible_entries()
                .iter()
                .enumerate()
                .find(|(index, _entry)| !ctx.tree.view.selected.contains(index))
                .map(|(index, _entry)| index)
        })
}

fn single_source_dest_cursor(ctx: &ReduceCtx<'_>) -> Option<usize> {
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

    Some(initial_cursor)
}
