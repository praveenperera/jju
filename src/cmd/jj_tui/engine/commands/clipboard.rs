use super::super::selection::get_revs_for_action;
use super::{Effect, ReduceCtx};
use crate::cmd::jj_tui::state::{
    ClipboardBranchOption, ClipboardBranchSelectState, MessageKind, ModeState,
};

const BRANCH_SELECTION_KEYS: &str = "abcdefghijklmnopqrstuvwxyz";

pub(super) fn copy_branch(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if node.bookmarks.is_empty() {
        ctx.set_status("No branch on this revision", MessageKind::Warning);
        return;
    }

    if node.bookmarks.len() == 1 {
        let branch = node.bookmarks[0].name.clone();
        ctx.effects.push(Effect::CopyToClipboard {
            value: branch.clone(),
            success: format!("Copied branch {branch} to clipboard"),
        });
        return;
    }

    if node.bookmarks.len() > BRANCH_SELECTION_KEYS.len() {
        ctx.set_status("Too many branches to copy", MessageKind::Warning);
        return;
    }

    let options = node
        .bookmarks
        .iter()
        .zip(BRANCH_SELECTION_KEYS.chars())
        .map(|(bookmark, key)| ClipboardBranchOption {
            key,
            branch: bookmark.name.clone(),
        })
        .collect();

    *ctx.mode = ModeState::ClipboardBranchSelect(ClipboardBranchSelectState {
        target_rev: node.change_id.clone(),
        options,
    });
}

pub(super) fn copy_commit_sha(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    ctx.effects.push(Effect::CopyToClipboard {
        value: node.commit_id.clone(),
        success: "Copied commit sha to clipboard".to_string(),
    });
}

pub(super) fn copy_rev(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    ctx.effects.push(Effect::CopyToClipboard {
        value: node.change_id.clone(),
        success: "Copied rev to clipboard".to_string(),
    });
}

pub(super) fn copy_commit_message(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    let fallback = node
        .details
        .as_ref()
        .map(|details| details.full_description.clone())
        .unwrap_or_else(|| node.description.clone());

    ctx.effects.push(Effect::CopyCommitMessageToClipboard {
        commit_id: node.commit_id.clone(),
        fallback,
    });
}

pub(super) fn copy_commit_subject(ctx: &mut ReduceCtx<'_>) {
    let Some(node) = ctx.tree.current_node() else {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    };

    if node.description.is_empty() {
        ctx.set_status("No commit subject on this revision", MessageKind::Warning);
        return;
    }

    ctx.effects.push(Effect::CopyToClipboard {
        value: node.description.clone(),
        success: "Copied commit subject to clipboard".to_string(),
    });
}

pub(super) fn copy_selection_revset(ctx: &mut ReduceCtx<'_>) {
    let revs = selected_revs_in_order(ctx);
    if revs.is_empty() || revs.iter().all(String::is_empty) {
        ctx.set_status("No revision selected", MessageKind::Error);
        return;
    }

    let revset = revs.join(" | ");
    ctx.effects.push(Effect::CopyToClipboard {
        value: revset,
        success: "Copied selection revset to clipboard".to_string(),
    });
}

fn selected_revs_in_order(ctx: &ReduceCtx<'_>) -> Vec<String> {
    if ctx.tree.view.selected.is_empty() {
        return get_revs_for_action(ctx.tree);
    }

    ctx.tree
        .visible_entries()
        .iter()
        .enumerate()
        .filter(|(index, _entry)| ctx.tree.view.selected.contains(index))
        .map(|(_index, entry)| ctx.tree.nodes()[entry.node_index].change_id.clone())
        .collect()
}
