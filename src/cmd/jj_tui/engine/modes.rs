use super::rebase::compute_moving_indices;
use super::selection::{current_rev, get_rev_at_cursor, get_revs_for_action};
use super::{Action, Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::handlers;
use crate::cmd::jj_tui::state::{
    ConfirmAction, ConfirmState, ConflictsState, DiffState, RebaseState, RebaseType, SquashState,
};
use jju_core::interactive::{InteractiveOperation, SquashOperation};

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EnterDiffView => enter_diff_view(ctx),
        Action::ExitDiffView => *ctx.mode = ModeState::Normal,
        Action::EnterConfirmStackSync => enter_confirm_stack_sync(ctx),
        Action::EnterConfirmAbandon => enter_confirm_abandon(ctx),
        Action::EnterConfirmRebaseOntoTrunk(rebase_type) => {
            enter_confirm_rebase_onto_trunk(ctx, rebase_type);
        }
        Action::ConfirmYes => confirm_yes(ctx),
        Action::ConfirmNo => *ctx.mode = ModeState::Normal,
        Action::EnterRebaseMode(rebase_type) => enter_rebase_mode(ctx, rebase_type),
        Action::ExitRebaseMode => *ctx.mode = ModeState::Normal,
        Action::MoveRebaseDestUp => move_rebase_dest_up(ctx),
        Action::MoveRebaseDestDown => move_rebase_dest_down(ctx),
        Action::ToggleRebaseBranches => toggle_rebase_branches(ctx),
        Action::ExecuteRebase => execute_rebase(ctx),
        Action::EnterSquashMode => enter_squash_mode(ctx),
        Action::ExitSquashMode => *ctx.mode = ModeState::Normal,
        Action::MoveSquashDestUp => move_squash_dest_up(ctx),
        Action::MoveSquashDestDown => move_squash_dest_down(ctx),
        Action::ExecuteSquash => execute_squash(ctx),
        Action::ScrollDiffUp(amount) => scroll_diff_up(ctx, amount),
        Action::ScrollDiffDown(amount) => scroll_diff_down(ctx, amount),
        Action::ScrollDiffTop => scroll_diff_top(ctx),
        Action::ScrollDiffBottom => scroll_diff_bottom(ctx),
        Action::EnterConflicts => {
            *ctx.mode = ModeState::Conflicts(ConflictsState::default());
            ctx.effects.push(Effect::LoadConflictFiles);
        }
        Action::ExitConflicts => *ctx.mode = ModeState::Normal,
        Action::ConflictsUp => conflicts_up(ctx),
        Action::ConflictsDown => conflicts_down(ctx),
        Action::ConflictsJump => *ctx.mode = ModeState::Normal,
        Action::StartResolveFromConflicts => start_resolve_from_conflicts(ctx),
        _ => unreachable!("unsupported mode action: {action:?}"),
    }
}

fn enter_diff_view(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    if let Ok(diff_output) = super::super::commands::diff::get_diff(&rev) {
        let lines = handlers::diff::parse_diff(&diff_output, ctx.syntax_set, ctx.theme_set);
        *ctx.mode = ModeState::ViewingDiff(DiffState {
            lines,
            scroll_offset: 0,
            rev,
        });
    }
}

fn enter_confirm_stack_sync(ctx: &mut ReduceCtx<'_>) {
    let trunk = super::super::commands::stack_sync::detect_trunk_branch()
        .unwrap_or_else(|_| "trunk".to_string());
    let roots = super::super::commands::stack_sync::find_stack_roots(&trunk).unwrap_or_default();

    let message = format!("Will rebase the following commits on top of {trunk}:");
    let mut revs = Vec::new();
    if roots.is_empty() {
        revs.push("  (stack is up to date, nothing to rebase)".to_string());
    } else {
        for root in &roots {
            let desc = super::super::commands::stack_sync::get_commit_description(root)
                .unwrap_or_default();
            revs.push(format!("  {root}  {desc}"));
            revs.push(format!(
                "  jj rebase --source (-s) {root} --onto (-o) {trunk} --skip-emptied"
            ));
        }
    }

    *ctx.mode = ModeState::Confirming(ConfirmState {
        action: ConfirmAction::StackSync,
        message,
        revs,
    });
}

fn enter_confirm_abandon(ctx: &mut ReduceCtx<'_>) {
    let revs = get_revs_for_action(ctx.tree);
    for rev in &revs {
        if ctx
            .tree
            .nodes
            .iter()
            .any(|node| node.change_id == *rev && node.is_working_copy)
        {
            ctx.set_status("Cannot abandon working copy", MessageKind::Error);
            return;
        }
    }

    let message = if revs.len() == 1 {
        format!("Abandon revision {}?", revs[0])
    } else {
        format!("Abandon {} revisions?", revs.len())
    };

    *ctx.mode = ModeState::Confirming(ConfirmState {
        action: ConfirmAction::Abandon,
        message,
        revs,
    });
}

fn enter_confirm_rebase_onto_trunk(ctx: &mut ReduceCtx<'_>, rebase_type: RebaseType) {
    let source = current_rev(ctx.tree);
    if source.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let short_rev = &source[..8.min(source.len())];
    let message = match rebase_type {
        RebaseType::Single => format!("Rebase {short_rev} onto trunk?"),
        RebaseType::WithDescendants => {
            format!("Rebase {short_rev} and descendants onto trunk?")
        }
    };

    let mode_flag = match rebase_type {
        RebaseType::Single => "-r",
        RebaseType::WithDescendants => "-s",
    };
    let cmd_preview = format!("jj rebase {mode_flag} {short_rev} -d trunk() --skip-emptied");

    *ctx.mode = ModeState::Confirming(ConfirmState {
        action: ConfirmAction::RebaseOntoTrunk(rebase_type),
        message,
        revs: vec![cmd_preview],
    });
}

fn confirm_yes(ctx: &mut ReduceCtx<'_>) {
    let ModeState::Confirming(state) = std::mem::replace(ctx.mode, ModeState::Normal) else {
        return;
    };

    match state.action {
        ConfirmAction::Abandon => {
            let revset = state.revs.join(" | ");
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects.push(Effect::RunAbandon { revset });
            ctx.effects.push(Effect::RefreshTree);
        }
        ConfirmAction::StackSync => {
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects.push(Effect::RunStackSync);
            ctx.effects.push(Effect::RefreshTree);
        }
        ConfirmAction::RebaseOntoTrunk(rebase_type) => {
            let source = current_rev(ctx.tree);
            if source.is_empty() {
                ctx.set_status("No revision selected", MessageKind::Error);
                return;
            }
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects.push(Effect::RunRebaseOntoTrunk {
                source,
                rebase_type,
            });
            ctx.effects.push(Effect::RefreshTree);
        }
        ConfirmAction::MoveBookmarkBackwards {
            bookmark_name,
            dest_rev,
        } => {
            ctx.effects.push(Effect::SaveOperationForUndo);
            ctx.effects.push(Effect::RunBookmarkSetBackwards {
                name: bookmark_name,
                rev: dest_rev,
            });
            ctx.effects.push(Effect::RefreshTree);
        }
    }

    ctx.tree.clear_selection();
}

fn enter_rebase_mode(ctx: &mut ReduceCtx<'_>, rebase_type: RebaseType) {
    let source_rev = current_rev(ctx.tree);
    if source_rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let current = ctx.tree.cursor;
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
        .visible_entries
        .get(current)
        .map(|entry| ctx.tree.nodes[entry.node_index].depth)
        .unwrap_or(0);

    let mut initial_cursor = current.saturating_sub(1);
    while initial_cursor > 0 {
        let entry = &ctx.tree.visible_entries[initial_cursor];
        let node = &ctx.tree.nodes[entry.node_index];
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

fn move_rebase_dest_up(ctx: &mut ReduceCtx<'_>) {
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

fn move_rebase_dest_down(ctx: &mut ReduceCtx<'_>) {
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

fn toggle_rebase_branches(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Rebasing(state) = ctx.mode {
        state.allow_branches = !state.allow_branches;
    }
}

fn execute_rebase(ctx: &mut ReduceCtx<'_>) {
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

fn enter_squash_mode(ctx: &mut ReduceCtx<'_>) {
    let source_rev = current_rev(ctx.tree);
    if source_rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let current = ctx.tree.cursor;
    let source_struct_depth = ctx
        .tree
        .visible_entries
        .get(current)
        .map(|entry| ctx.tree.nodes[entry.node_index].depth)
        .unwrap_or(0);

    let mut initial_cursor = current.saturating_sub(1);
    while initial_cursor > 0 {
        let entry = &ctx.tree.visible_entries[initial_cursor];
        let node = &ctx.tree.nodes[entry.node_index];
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

fn move_squash_dest_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Squashing(state) = ctx.mode
        && state.dest_cursor > 0
    {
        state.dest_cursor -= 1;
    }
}

fn move_squash_dest_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Squashing(state) = ctx.mode {
        let max = ctx.tree.visible_count().saturating_sub(1);
        if state.dest_cursor < max {
            state.dest_cursor += 1;
        }
    }
}

fn execute_squash(ctx: &mut ReduceCtx<'_>) {
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

fn scroll_diff_up(ctx: &mut ReduceCtx<'_>, amount: usize) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = state.scroll_offset.saturating_sub(amount);
    }
}

fn scroll_diff_down(ctx: &mut ReduceCtx<'_>, amount: usize) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = state.scroll_offset.saturating_add(amount);
    }
}

fn scroll_diff_top(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = 0;
    }
}

fn scroll_diff_bottom(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::ViewingDiff(state) = ctx.mode {
        state.scroll_offset = state.lines.len().saturating_sub(1);
    }
}

fn conflicts_up(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Conflicts(state) = ctx.mode
        && state.selected_index > 0
    {
        state.selected_index -= 1;
    }
}

fn conflicts_down(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Conflicts(state) = ctx.mode {
        let max = state.files.len().saturating_sub(1);
        if state.selected_index < max {
            state.selected_index += 1;
        }
    }
}

fn start_resolve_from_conflicts(ctx: &mut ReduceCtx<'_>) {
    if let ModeState::Conflicts(state) = ctx.mode
        && let Some(file) = state.files.get(state.selected_index).cloned()
    {
        ctx.effects
            .push(Effect::RunInteractive(InteractiveOperation::Resolve {
                file,
            }));
        *ctx.mode = ModeState::Normal;
    }
}
