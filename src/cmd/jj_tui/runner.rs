//! Effects runner for jj_tui
//!
//! The runner executes effects by performing IO operations.
//! It handles terminal restore/init for operations that need the terminal.

use super::commands;
use super::effect::Effect;
use super::state::{DiffStats, MessageKind, RebaseType};
use super::tree::TreeState;
use crate::jj_lib_helpers::JjRepo;
use ratatui::DefaultTerminal;
use std::fs;
use std::io::Write;
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
                if let Err(e) = refresh_tree(tree, diff_stats_cache) {
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
                        set_error_with_details("Abandon failed", &error_details),
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
                let res = match (rebase_type, allow_branches) {
                    // fork: -d flag, dest's children unaffected
                    (RebaseType::Single, true) => {
                        commands::rebase::single_fork(&source, &dest)
                    }
                    (RebaseType::WithDescendants, true) => {
                        commands::rebase::with_descendants_fork(&source, &dest)
                    }
                    // inline: -A flag, dest's children reparented under source
                    (RebaseType::Single, false) => {
                        commands::rebase::single(&source, &dest)
                    }
                    (RebaseType::WithDescendants, false) => {
                        commands::rebase::with_descendants(&source, &dest)
                    }
                };

                match res {
                    Ok(_) => {
                        let has_conflicts = commands::has_conflicts().unwrap_or(false);
                        if has_conflicts {
                            result.status_message = Some((
                                "Rebase created conflicts. Press u to undo".to_string(),
                                MessageKind::Warning,
                            ));
                        } else {
                            result.status_message =
                                Some(("Rebase complete".to_string(), MessageKind::Success));
                        }
                    }
                    Err(e) => {
                        result.status_message =
                            Some((format!("Rebase failed: {e}"), MessageKind::Error));
                    }
                }
            }

            Effect::RunRebaseOntoTrunk {
                source,
                rebase_type,
            } => {
                let res = match rebase_type {
                    RebaseType::Single => commands::rebase::single_onto_trunk(&source),
                    RebaseType::WithDescendants => {
                        commands::rebase::with_descendants_onto_trunk(&source)
                    }
                };

                match res {
                    Ok(_) => {
                        let has_conflicts = commands::has_conflicts().unwrap_or(false);
                        if has_conflicts {
                            result.status_message = Some((
                                "Rebased onto trunk (conflicts detected, u to undo)".to_string(),
                                MessageKind::Warning,
                            ));
                        } else {
                            result.status_message =
                                Some(("Rebased onto trunk".to_string(), MessageKind::Success));
                        }
                    }
                    Err(e) => {
                        let error_details = format!("{e}");
                        result.status_message = Some((
                            set_error_with_details("Rebase failed", &error_details),
                            MessageKind::Error,
                        ));
                    }
                }
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
                if let Some(ref op_id) = last_op.take() {
                    match commands::restore_op(op_id) {
                        Ok(_) => {
                            result.status_message =
                                Some(("Operation undone".to_string(), MessageKind::Success));
                        }
                        Err(e) => {
                            result.status_message =
                                Some((format!("Undo failed: {e}"), MessageKind::Error));
                        }
                    }
                } else {
                    result.status_message =
                        Some(("Nothing to undo".to_string(), MessageKind::Warning));
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
                let sync_result = (|| -> eyre::Result<String> {
                    commands::git::fetch()
                        .map_err(|e| eyre::eyre!("Stack sync failed (fetch): {e}"))?;

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

                    let deleted =
                        commands::stack_sync::cleanup_deleted_bookmarks().unwrap_or_default();

                    let has_conflicts = commands::has_conflicts().unwrap_or(false);
                    if has_conflicts {
                        return Ok("Stack synced (conflicts detected, u to undo)".to_string());
                    }

                    let mut msg = format!("Stack synced onto {trunk}");
                    if !deleted.is_empty() {
                        msg.push_str(&format!(
                            ", cleaned up {} bookmark{}",
                            deleted.len(),
                            if deleted.len() == 1 { "" } else { "s" }
                        ));
                    }
                    Ok(msg)
                })();

                match sync_result {
                    Ok(msg) => {
                        let kind = if msg.contains("conflicts") {
                            MessageKind::Warning
                        } else {
                            MessageKind::Success
                        };
                        result.status_message = Some((msg, kind));
                    }
                    Err(e) => {
                        let error_details = format!("{e}");
                        result.status_message = Some((
                            set_error_with_details("Stack sync failed", &error_details),
                            MessageKind::Error,
                        ));
                    }
                }
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

            Effect::RunBookmarkSet { name, rev } => match commands::bookmark::set(&name, &rev) {
                Ok(_) => {
                    let short_rev = &rev[..8.min(rev.len())];
                    result.status_message = Some((
                        format!("Moved bookmark '{}' to {}", name, short_rev),
                        MessageKind::Success,
                    ));
                }
                Err(e) => {
                    let error_details = format!("{e}");
                    result.status_message = Some((
                        set_error_with_details("Move bookmark failed", &error_details),
                        MessageKind::Error,
                    ));
                }
            },

            Effect::RunBookmarkSetBackwards { name, rev } => {
                match commands::bookmark::set_allow_backwards(&name, &rev) {
                    Ok(_) => {
                        let short_rev = &rev[..8.min(rev.len())];
                        result.status_message = Some((
                            format!("Moved bookmark '{}' to {}", name, short_rev),
                            MessageKind::Success,
                        ));
                    }
                    Err(e) => {
                        let error_details = format!("{e}");
                        result.status_message = Some((
                            set_error_with_details("Move bookmark failed", &error_details),
                            MessageKind::Error,
                        ));
                    }
                }
            }

            Effect::RunBookmarkDelete { name } => match commands::bookmark::delete(&name) {
                Ok(_) => {
                    result.status_message =
                        Some((format!("Deleted bookmark '{name}'"), MessageKind::Success));
                }
                Err(e) => {
                    let error_details = format!("{e}");
                    result.status_message = Some((
                        set_error_with_details("Delete bookmark failed", &error_details),
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
                            set_error_with_details("Resolve divergence failed", &error_details),
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

/// Refresh the tree state
fn refresh_tree(
    tree: &mut TreeState,
    diff_stats_cache: &mut std::collections::HashMap<String, DiffStats>,
) -> eyre::Result<()> {
    // save current position to restore after refresh
    let current_change_id = tree.current_node().map(|n| n.change_id.clone());
    let parent_change_id = tree
        .current_node()
        .and_then(|n| n.parent_ids.first().cloned());
    let old_cursor = tree.cursor;

    // save focus stack change_ids to restore after refresh
    let focus_stack_change_ids: Vec<String> = tree
        .focus_stack
        .iter()
        .filter_map(|&idx| tree.nodes.get(idx).map(|n| n.change_id.clone()))
        .collect();

    let jj_repo = JjRepo::load(None)?;
    *tree = TreeState::load(&jj_repo)?;
    tree.clear_selection();
    diff_stats_cache.clear();

    // restore focus stack if the focused nodes still exist
    for change_id in focus_stack_change_ids {
        if let Some(node_idx) = tree.nodes.iter().position(|n| n.change_id == change_id) {
            tree.focus_on(node_idx);
        }
    }

    // restore cursor: try current change_id, then parent, then clamp old position
    let find_visible = |cid: &str| {
        tree.visible_entries
            .iter()
            .position(|e| tree.nodes[e.node_index].change_id == cid)
    };

    if let Some(ref cid) = current_change_id
        && let Some(idx) = find_visible(cid)
    {
        tree.cursor = idx;
    } else if let Some(ref pid) = parent_change_id
        && let Some(idx) = find_visible(pid)
    {
        tree.cursor = idx;
    } else {
        tree.cursor = old_cursor.min(tree.visible_count().saturating_sub(1));
    }

    Ok(())
}

/// Save error details to a temp file and return a formatted error message
fn set_error_with_details(prefix: &str, stderr: &str) -> String {
    let first_line = stderr.lines().next().unwrap_or(stderr);
    let truncated = if first_line.len() > 80 {
        format!("{}...", &first_line[..77])
    } else {
        first_line.to_string()
    };

    if let Some(path) = save_error_to_file(stderr) {
        format!("{prefix}: {truncated} (full error: {path})")
    } else {
        format!("{prefix}: {truncated}")
    }
}

/// Save error details to a temp file and return the path
fn save_error_to_file(error: &str) -> Option<String> {
    let temp_dir = std::env::temp_dir();
    let error_file = temp_dir.join(format!("jju-error-{}.log", std::process::id()));
    let path = error_file.to_string_lossy().to_string();

    match fs::File::create(&error_file) {
        Ok(mut file) => {
            if file.write_all(error.as_bytes()).is_ok() {
                Some(path)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
