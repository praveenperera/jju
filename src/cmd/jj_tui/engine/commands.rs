use super::selection::current_rev;
use super::{Action, Effect, MessageKind, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::BookmarkSelectAction;
use jju_core::interactive::InteractiveOperation;

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EditWorkingCopy => edit_working_copy(ctx),
        Action::CreateNewCommit => create_new_commit(ctx),
        Action::CommitWorkingCopy => commit_working_copy(ctx),
        Action::EditDescription => edit_description(ctx),
        Action::Undo => {
            ctx.effects.push(Effect::RunUndo);
            ctx.effects.push(Effect::RefreshTree);
        }
        Action::GitFetch => run_simple_refresh(ctx, Effect::RunGitFetch),
        Action::GitImport => run_simple_refresh(ctx, Effect::RunGitImport),
        Action::GitExport => run_simple_refresh(ctx, Effect::RunGitExport),
        Action::ResolveDivergence => resolve_divergence(ctx),
        Action::CreatePR => create_pr(ctx),
        _ => unreachable!("unsupported command action: {action:?}"),
    }
}

fn edit_working_copy(ctx: &mut ReduceCtx<'_>) {
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

fn create_new_commit(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    ctx.effects.push(Effect::RunNew { rev });
    ctx.effects.push(Effect::RefreshTree);
}

fn commit_working_copy(ctx: &mut ReduceCtx<'_>) {
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

fn edit_description(ctx: &mut ReduceCtx<'_>) {
    let rev = current_rev(ctx.tree);
    if rev.is_empty() {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    ctx.effects.push(Effect::RunInteractive(
        InteractiveOperation::EditDescription { rev },
    ));
}

fn run_simple_refresh(ctx: &mut ReduceCtx<'_>, effect: Effect) {
    ctx.effects.push(effect);
    ctx.effects.push(Effect::RefreshTree);
}

fn resolve_divergence(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if !node.is_divergent {
        ctx.set_status("This revision is not divergent", MessageKind::Warning);
        return;
    }

    if node.divergent_versions.is_empty() {
        ctx.set_status("No divergent versions found", MessageKind::Error);
        return;
    }

    let local_version = node
        .divergent_versions
        .iter()
        .find(|version| version.is_local)
        .unwrap_or(&node.divergent_versions[0]);

    let abandon_ids: Vec<String> = node
        .divergent_versions
        .iter()
        .filter(|version| version.commit_id != local_version.commit_id)
        .map(|version| version.commit_id.clone())
        .collect();

    if abandon_ids.is_empty() {
        ctx.set_status(
            "Only one version exists, nothing to resolve",
            MessageKind::Warning,
        );
        return;
    }

    ctx.effects.push(Effect::SaveOperationForUndo);
    ctx.effects.push(Effect::RunResolveDivergence {
        keep_commit_id: local_version.commit_id.clone(),
        abandon_commit_ids: abandon_ids,
    });
    ctx.effects.push(Effect::RefreshTree);
}

fn create_pr(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if node.bookmarks.is_empty() {
        ctx.set_status(
            "No bookmark on this revision to create PR from",
            MessageKind::Warning,
        );
        return;
    }

    if node.bookmarks.len() == 1 {
        let bookmark = node.bookmarks[0].name.clone();
        ctx.effects.push(Effect::RunCreatePR { bookmark });
        return;
    }

    let bookmarks: Vec<String> = node
        .bookmarks
        .iter()
        .map(|bookmark| bookmark.name.clone())
        .collect();
    let target_rev = node.change_id.clone();
    *ctx.mode = ModeState::BookmarkSelect(super::super::state::BookmarkSelectState {
        bookmarks,
        selected_index: 0,
        target_rev,
        action: BookmarkSelectAction::CreatePR,
    });
}
