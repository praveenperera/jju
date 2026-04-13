use super::super::super::selection::{current_rev, get_revs_for_action};
use super::super::{ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::{ConfirmAction, ConfirmState, MessageKind, RebaseType};

pub(super) fn stack_sync(ctx: &mut ReduceCtx<'_>) {
    let trunk = super::super::super::super::commands::stack_sync::detect_trunk_branch()
        .unwrap_or_else(|_| "trunk".to_string());
    let roots = super::super::super::super::commands::stack_sync::find_stack_roots(&trunk)
        .unwrap_or_default();

    *ctx.mode = ModeState::Confirming(ConfirmState {
        action: ConfirmAction::StackSync,
        message: format!("Will rebase the following commits on top of {trunk}:"),
        revs: stack_sync_revs(&trunk, &roots),
    });
}

pub(super) fn abandon(ctx: &mut ReduceCtx<'_>) {
    let revs = get_revs_for_action(ctx.tree);
    if revs.iter().any(|rev| is_working_copy(ctx, rev)) {
        ctx.set_status("Cannot abandon working copy", MessageKind::Error);
        return;
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

pub(super) fn rebase_onto_trunk(ctx: &mut ReduceCtx<'_>, rebase_type: RebaseType) {
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

    *ctx.mode = ModeState::Confirming(ConfirmState {
        action: ConfirmAction::RebaseOntoTrunk(rebase_type),
        message,
        revs: vec![command_preview(short_rev, rebase_type)],
    });
}

fn stack_sync_revs(trunk: &str, roots: &[String]) -> Vec<String> {
    if roots.is_empty() {
        return vec!["  (stack is up to date, nothing to rebase)".to_string()];
    }

    let mut revs = Vec::new();
    for root in roots {
        let desc = super::super::super::super::commands::stack_sync::get_commit_description(root)
            .unwrap_or_default();
        revs.push(format!("  {root}  {desc}"));
        revs.push(format!(
            "  jj rebase --source (-s) {root} --onto (-o) {trunk} --skip-emptied"
        ));
    }
    revs
}

fn is_working_copy(ctx: &ReduceCtx<'_>, rev: &str) -> bool {
    ctx.tree
        .nodes()
        .iter()
        .any(|node| node.change_id == *rev && node.is_working_copy)
}

fn command_preview(short_rev: &str, rebase_type: RebaseType) -> String {
    let mode_flag = match rebase_type {
        RebaseType::Single => "-r",
        RebaseType::WithDescendants => "-s",
    };
    format!("jj rebase {mode_flag} {short_rev} -d trunk() --skip-emptied")
}
