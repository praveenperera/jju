use super::super::selection::{current_rev, get_revs_for_action};
use super::super::{Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::{ConfirmAction, ConfirmState, RebaseType};

pub(super) fn enter_stack_sync(ctx: &mut ReduceCtx<'_>) {
    let trunk = super::super::super::commands::stack_sync::detect_trunk_branch()
        .unwrap_or_else(|_| "trunk".to_string());
    let roots =
        super::super::super::commands::stack_sync::find_stack_roots(&trunk).unwrap_or_default();

    let message = format!("Will rebase the following commits on top of {trunk}:");
    let mut revs = Vec::new();
    if roots.is_empty() {
        revs.push("  (stack is up to date, nothing to rebase)".to_string());
    } else {
        for root in &roots {
            let desc = super::super::super::commands::stack_sync::get_commit_description(root)
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

pub(super) fn enter_abandon(ctx: &mut ReduceCtx<'_>) {
    let revs = get_revs_for_action(ctx.tree);
    for rev in &revs {
        if ctx
            .tree
            .nodes()
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

pub(super) fn enter_rebase_onto_trunk(ctx: &mut ReduceCtx<'_>, rebase_type: RebaseType) {
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

pub(super) fn confirm_yes(ctx: &mut ReduceCtx<'_>) {
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
