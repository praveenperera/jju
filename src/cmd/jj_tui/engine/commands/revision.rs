use super::super::selection::current_rev;
use super::super::{Effect, MessageKind, ReduceCtx};
use jju_core::interactive::InteractiveOperation;

pub(super) fn edit_working_copy(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    if let Some(node) = ctx.tree.current_node()
        && node.is_working_copy
    {
        ctx.set_status("Already editing this revision", MessageKind::Warning);
        return;
    }

    ctx.effects.push(Effect::RunEdit { rev });
    ctx.effects.push(Effect::RefreshTree);
}

pub(super) fn create_new_commit(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    ctx.effects.push(Effect::RunNew { rev });
    ctx.effects.push(Effect::RefreshTree);
}

pub(super) fn commit_working_copy(ctx: &mut ReduceCtx<'_>) {
    if let Some(node) = ctx.tree.current_node()
        && !node.is_working_copy
    {
        ctx.set_status(
            "Can only commit from working copy (@)",
            MessageKind::Warning,
        );
        return;
    }

    if let Some(node) = ctx.tree.current_node() {
        let message = if node.description.is_empty() {
            "(no description)".to_string()
        } else {
            node.description.clone()
        };
        ctx.effects.push(Effect::RunCommit { message });
        ctx.effects.push(Effect::RefreshTree);
    }
}

pub(super) fn edit_description(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    ctx.effects.push(Effect::RunInteractive(
        InteractiveOperation::EditDescription { rev },
    ));
}
