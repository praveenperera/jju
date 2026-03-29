use super::error::set_error_with_details;
use crate::cmd::jj_tui::commands;
use crate::cmd::jj_tui::state::{MessageKind, RebaseType};

pub fn run_rebase(
    source: &str,
    dest: &str,
    rebase_type: RebaseType,
    allow_branches: bool,
) -> (String, MessageKind) {
    let result = match (rebase_type, allow_branches) {
        (RebaseType::Single, true) => commands::rebase::single_fork(source, dest),
        (RebaseType::WithDescendants, true) => {
            commands::rebase::with_descendants_fork(source, dest)
        }
        (RebaseType::Single, false) => commands::rebase::single(source, dest),
        (RebaseType::WithDescendants, false) => commands::rebase::with_descendants(source, dest),
    };

    match result {
        Ok(_) => conflict_aware_status(
            "Rebase complete",
            "Rebase created conflicts. Press u to undo",
        ),
        Err(error) => (format!("Rebase failed: {error}"), MessageKind::Error),
    }
}

pub fn run_rebase_onto_trunk(source: &str, rebase_type: RebaseType) -> (String, MessageKind) {
    let result = match rebase_type {
        RebaseType::Single => commands::rebase::single_onto_trunk(source),
        RebaseType::WithDescendants => commands::rebase::with_descendants_onto_trunk(source),
    };

    match result {
        Ok(_) => conflict_aware_status(
            "Rebased onto trunk",
            "Rebased onto trunk (conflicts detected, u to undo)",
        ),
        Err(error) => (
            set_error_with_details("Rebase failed", &error.to_string()),
            MessageKind::Error,
        ),
    }
}

pub fn run_stack_sync() -> (String, MessageKind) {
    let result = (|| -> eyre::Result<String> {
        commands::git::fetch().map_err(|e| eyre::eyre!("Stack sync failed (fetch): {e}"))?;

        let trunk = commands::stack_sync::detect_trunk_branch()
            .map_err(|e| eyre::eyre!("Stack sync failed (detect trunk): {e}"))?;

        commands::stack_sync::sync_trunk_bookmark(&trunk)
            .map_err(|e| eyre::eyre!("Stack sync failed (sync trunk): {e}"))?;

        let roots = commands::stack_sync::find_stack_roots(&trunk)
            .map_err(|e| eyre::eyre!("Stack sync failed (find roots): {e}"))?;

        if roots.is_empty() {
            return Ok("Nothing to rebase, stack is up to date".to_string());
        }

        for root in &roots {
            commands::stack_sync::rebase_root_onto_trunk(root, &trunk)
                .map_err(|e| eyre::eyre!("Stack sync failed (rebase {root}): {e}"))?;
        }

        let deleted = commands::stack_sync::cleanup_deleted_bookmarks().unwrap_or_default();

        if commands::has_conflicts().unwrap_or(false) {
            return Ok("Stack synced (conflicts detected, u to undo)".to_string());
        }

        let mut message = format!("Stack synced onto {trunk}");
        if !deleted.is_empty() {
            message.push_str(&format!(
                ", cleaned up {} bookmark{}",
                deleted.len(),
                if deleted.len() == 1 { "" } else { "s" }
            ));
        }
        Ok(message)
    })();

    match result {
        Ok(message) => {
            let kind = if message.contains("conflicts") {
                MessageKind::Warning
            } else {
                MessageKind::Success
            };
            (message, kind)
        }
        Err(error) => (
            set_error_with_details("Stack sync failed", &error.to_string()),
            MessageKind::Error,
        ),
    }
}

pub fn run_bookmark_set(name: &str, rev: &str) -> (String, MessageKind) {
    bookmark_result(
        commands::bookmark::set(name, rev),
        "Move bookmark failed",
        name,
        rev,
    )
}

pub fn run_bookmark_set_backwards(name: &str, rev: &str) -> (String, MessageKind) {
    bookmark_result(
        commands::bookmark::set_allow_backwards(name, rev),
        "Move bookmark failed",
        name,
        rev,
    )
}

fn bookmark_result(
    result: eyre::Result<()>,
    error_prefix: &str,
    name: &str,
    rev: &str,
) -> (String, MessageKind) {
    match result {
        Ok(_) => {
            let short_rev = &rev[..8.min(rev.len())];
            (
                format!("Moved bookmark '{name}' to {short_rev}"),
                MessageKind::Success,
            )
        }
        Err(error) => (
            set_error_with_details(error_prefix, &error.to_string()),
            MessageKind::Error,
        ),
    }
}

fn conflict_aware_status(success: &str, conflict: &str) -> (String, MessageKind) {
    if commands::has_conflicts().unwrap_or(false) {
        (conflict.to_string(), MessageKind::Warning)
    } else {
        (success.to_string(), MessageKind::Success)
    }
}
