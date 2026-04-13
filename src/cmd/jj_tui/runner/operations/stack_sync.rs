use crate::cmd::jj_tui::commands;
use crate::cmd::jj_tui::runner::error::set_error_with_details;
use crate::cmd::jj_tui::state::MessageKind;

pub(super) fn run_stack_sync() -> (String, MessageKind) {
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

        Ok(success_message(&trunk, &deleted))
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

fn success_message(trunk: &str, deleted: &[String]) -> String {
    let mut message = format!("Stack synced onto {trunk}");
    if !deleted.is_empty() {
        message.push_str(&format!(
            ", cleaned up {} bookmark{}",
            deleted.len(),
            if deleted.len() == 1 { "" } else { "s" }
        ));
    }
    message
}
