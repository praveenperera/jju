use super::RunCtx;
use jju_core::interactive::{InteractiveOperation, SquashOperation};
use jju_jj::ops::ConflictOps;
use ratatui::DefaultTerminal;
use std::process::Command;

pub(super) fn handle(
    ctx: &mut RunCtx<'_>,
    terminal: &mut DefaultTerminal,
    operation: InteractiveOperation,
) {
    match operation {
        InteractiveOperation::EditDescription { rev } => {
            ratatui::restore();
            let status = Command::new("jj").args(["describe", "-r", &rev]).status();
            *terminal = ratatui::init();
            match status {
                Ok(exit_status) if exit_status.success() => {
                    ctx.success("Description updated");
                    ctx.refresh_tree();
                }
                Ok(_) => ctx.warn("Editor cancelled"),
                Err(error) => ctx.error(format!("Failed to launch editor: {error}")),
            }
        }
        InteractiveOperation::Squash(squash) => handle_squash(ctx, terminal, squash),
        InteractiveOperation::Resolve { file } => {
            ratatui::restore();
            let result = ConflictOps.resolve_file(&file);
            *terminal = ratatui::init();

            match result {
                Ok(()) => {
                    let has_conflicts = ConflictOps.has_conflicts().unwrap_or(false);
                    ctx.refresh_tree();
                    if has_conflicts {
                        ctx.warn(format!("Resolved {file}. More conflicts remain"));
                    } else {
                        ctx.success("All conflicts resolved");
                    }
                }
                Err(error) => ctx.error(format!("Resolve failed: {error}")),
            }
        }
    }
}

fn handle_squash(ctx: &mut RunCtx<'_>, terminal: &mut DefaultTerminal, squash: SquashOperation) {
    ratatui::restore();
    let source_revset = squash.source_revs.join(" | ");
    let status = Command::new("jj")
        .args(["squash", "-f", &source_revset, "-t", &squash.target_rev])
        .status();
    *terminal = ratatui::init();

    match status {
        Ok(exit_status) if exit_status.success() => {
            *ctx.last_op = Some(squash.op_before);
            let has_conflicts = ConflictOps.has_conflicts().unwrap_or(false);
            ctx.refresh_tree();
            if has_conflicts {
                ctx.warn("Squash created conflicts. Press u to undo");
            } else {
                ctx.success("Squash complete");
            }
        }
        Ok(_) => ctx.warn("Squash cancelled"),
        Err(error) => ctx.error(format!("Squash failed: {error}")),
    }
}
