//! Effects runner for jj_tui
//!
//! The runner executes effects by performing IO operations.
//! It handles terminal restore/init for operations that need the terminal.

mod error;
mod operations;

use super::commands;
use super::effect::Effect;
use super::refresh;
use super::state::{DiffStats, MessageKind};
use super::tree::TreeState;
use ratatui::DefaultTerminal;
use std::time::Duration;

/// Result of running effects
#[derive(Default)]
pub struct RunResult {
    pub status_message: Option<(String, MessageKind)>,
    pub status_duration: Option<Duration>,
}

/// Execute a list of effects
pub fn run_effects(
    effects: Vec<Effect>,
    tree: &mut TreeState,
    diff_stats_cache: &mut std::collections::HashMap<String, DiffStats>,
    last_op: &mut Option<String>,
    _terminal: &mut DefaultTerminal,
) -> RunResult {
    let mut result = RunResult::default();

    for effect in effects {
        match effect {
            Effect::Quit => {
                // handled by should_quit flag
            }

            Effect::RefreshTree => {
                if let Err(e) = refresh::refresh_tree(tree, diff_stats_cache) {
                    result.status_message =
                        Some((format!("Failed to refresh: {e}"), MessageKind::Error));
                }
            }

            Effect::SaveOperationForUndo => {
                if let Ok(op_id) = commands::get_current_op_id() {
                    *last_op = Some(op_id);
                }
            }

            Effect::RunEdit { rev } => match commands::revision::edit(&rev) {
                Ok(_) => {
                    result.status_message =
                        Some((format!("Now editing {rev}"), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message = Some((format!("Edit failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunNew { rev } => match commands::revision::new(&rev) {
                Ok(_) => {
                    result.status_message =
                        Some(("Created new commit".to_string(), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message = Some((format!("Failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunCommit { message } => match commands::revision::commit(&message) {
                Ok(_) => {
                    result.status_message =
                        Some(("Changes committed".to_string(), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message =
                        Some((format!("Commit failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunAbandon { revset } => match commands::revision::abandon(&revset) {
                Ok(_) => {
                    let count = revset.matches('|').count() + 1;
                    let msg = if count == 1 {
                        "Revision abandoned".to_string()
                    } else {
                        format!("{} revisions abandoned", count)
                    };
                    result.status_message = Some((msg, MessageKind::Success));
                }
                Err(e) => {
                    let error_details = format!("{e}");
                    result.status_message = Some((
                        error::set_error_with_details("Abandon failed", &error_details),
                        MessageKind::Error,
                    ));
                }
            },

            Effect::RunRebase {
                source,
                dest,
                rebase_type,
                allow_branches,
            } => {
                result.status_message = Some(operations::run_rebase(
                    &source,
                    &dest,
                    rebase_type,
                    allow_branches,
                ));
            }

            Effect::RunRebaseOntoTrunk {
                source,
                rebase_type,
            } => {
                result.status_message =
                    Some(operations::run_rebase_onto_trunk(&source, rebase_type));
            }

            Effect::RunSquash { source, target } => {
                // this should be handled via pending_operation in the main loop
                // because it requires terminal restore for the editor
                result.status_message = Some((
                    format!("Squash {} into {}", source, target),
                    MessageKind::Success,
                ));
            }

            Effect::RunUndo { op_id: _ } => {
                // use last_op from state, not from effect
                match last_op.take() {
                    Some(ref op_id) if !op_id.is_empty() => match commands::restore_op(op_id) {
                        Ok(_) => {
                            result.status_message =
                                Some(("Operation undone".to_string(), MessageKind::Success));
                        }
                        Err(e) => {
                            result.status_message =
                                Some((format!("Undo failed: {e}"), MessageKind::Error));
                        }
                    },
                    _ => {
                        result.status_message =
                            Some(("Nothing to undo".to_string(), MessageKind::Warning));
                    }
                }
            }

            Effect::RunGitPush { bookmark } => match commands::git::push_bookmark(&bookmark) {
                Ok(_) => {
                    result.status_message = Some((
                        format!("Pushed bookmark '{bookmark}'"),
                        MessageKind::Success,
                    ));
                }
                Err(e) => {
                    result.status_message = Some((format!("Push failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunGitPushMultiple { bookmarks } => {
                let mut succeeded = Vec::new();
                let mut failed = Vec::new();

                for bookmark in bookmarks {
                    match commands::git::push_bookmark(&bookmark) {
                        Ok(_) => succeeded.push(bookmark),
                        Err(e) => failed.push((bookmark, e.to_string())),
                    }
                }

                if failed.is_empty() {
                    let msg = if succeeded.len() == 1 {
                        format!("Pushed bookmark '{}'", succeeded[0])
                    } else {
                        format!("Pushed {} bookmarks", succeeded.len())
                    };
                    result.status_message = Some((msg, MessageKind::Success));
                } else if succeeded.is_empty() {
                    let first_err = &failed[0];
                    let msg = if failed.len() == 1 {
                        format!("Push failed for '{}': {}", first_err.0, first_err.1)
                    } else {
                        format!("Push failed for {} bookmarks", failed.len())
                    };
                    result.status_message = Some((msg, MessageKind::Error));
                } else {
                    let msg = format!(
                        "Pushed {} bookmarks, {} failed",
                        succeeded.len(),
                        failed.len()
                    );
                    result.status_message = Some((msg, MessageKind::Warning));
                }
            }

            Effect::RunGitPushAll => match commands::git::push_all() {
                Ok(_) => {
                    result.status_message =
                        Some(("Pushed all bookmarks".to_string(), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message =
                        Some((format!("Push all failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunStackSync => {
                result.status_message = Some(operations::run_stack_sync());
            }

            Effect::RunGitFetch => match commands::git::fetch() {
                Ok(_) => {
                    result.status_message =
                        Some(("Git fetch complete".to_string(), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message =
                        Some((format!("Git fetch failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunGitImport => match commands::git::import() {
                Ok(_) => {
                    result.status_message =
                        Some(("Git import complete".to_string(), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message =
                        Some((format!("Git import failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunGitExport => match commands::git::export() {
                Ok(_) => {
                    result.status_message =
                        Some(("Git export complete".to_string(), MessageKind::Success));
                }
                Err(e) => {
                    result.status_message =
                        Some((format!("Git export failed: {e}"), MessageKind::Error));
                }
            },

            Effect::RunBookmarkSet { name, rev } => {
                result.status_message = Some(operations::run_bookmark_set(&name, &rev));
            }

            Effect::RunBookmarkSetBackwards { name, rev } => {
                result.status_message = Some(operations::run_bookmark_set_backwards(&name, &rev));
            }

            Effect::RunBookmarkDelete { name } => match commands::bookmark::delete(&name) {
                Ok(_) => {
                    result.status_message =
                        Some((format!("Deleted bookmark '{name}'"), MessageKind::Success));
                }
                Err(e) => {
                    let error_details = format!("{e}");
                    result.status_message = Some((
                        error::set_error_with_details("Delete bookmark failed", &error_details),
                        MessageKind::Error,
                    ));
                }
            },

            Effect::RunResolveDivergence {
                keep_commit_id,
                abandon_commit_ids,
            } => {
                // abandon all the other versions to resolve divergence
                let revset = abandon_commit_ids.join(" | ");
                match commands::revision::abandon(&revset) {
                    Ok(_) => {
                        let count = abandon_commit_ids.len();
                        let short_keep = &keep_commit_id[..keep_commit_id.len().min(8)];
                        result.status_message = Some((
                            format!(
                                "Divergence resolved: kept {}, abandoned {} version{}",
                                short_keep,
                                count,
                                if count == 1 { "" } else { "s" }
                            ),
                            MessageKind::Success,
                        ));
                    }
                    Err(e) => {
                        let error_details = format!("{e}");
                        result.status_message = Some((
                            error::set_error_with_details(
                                "Resolve divergence failed",
                                &error_details,
                            ),
                            MessageKind::Error,
                        ));
                    }
                }
            }

            Effect::RunCreatePR { bookmark } => match commands::git::push_and_pr(&bookmark) {
                Ok(true) => {
                    result.status_message = Some((
                        format!("Pushed '{bookmark}' and opened PR"),
                        MessageKind::Success,
                    ));
                }
                Ok(false) => {
                    result.status_message = Some((
                        format!("Pushed '{bookmark}' and opened PR creation"),
                        MessageKind::Success,
                    ));
                }
                Err(e) => {
                    result.status_message = Some((format!("PR failed: {e}"), MessageKind::Error));
                }
            },

            Effect::LaunchDescriptionEditor { rev: _ } => {
                // handled via pending_operation in the main loop
            }

            Effect::LaunchSquashEditor {
                source: _,
                target: _,
                op_before: _,
            } => {
                // handled via pending_operation in the main loop
            }

            Effect::SetStatus { text, kind } => {
                result.status_message = Some((text, kind));
            }

            Effect::LoadConflictFiles => {
                // handled specially by the app - update mode state with conflict files
            }
        }
    }

    result
}
