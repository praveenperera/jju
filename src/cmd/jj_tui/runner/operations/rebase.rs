use crate::cmd::jj_tui::commands;
use crate::cmd::jj_tui::runner::error::set_error_with_details;
use crate::cmd::jj_tui::state::{MessageKind, RebaseType};

pub(super) fn run_rebase(
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

pub(super) fn run_rebase_onto_trunk(
    source: &str,
    rebase_type: RebaseType,
) -> (String, MessageKind) {
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

fn conflict_aware_status(success: &str, conflict: &str) -> (String, MessageKind) {
    if commands::has_conflicts().unwrap_or(false) {
        (conflict.to_string(), MessageKind::Warning)
    } else {
        (success.to_string(), MessageKind::Success)
    }
}
