//! Effects runner for jj_tui
//!
//! The runner executes effects by performing IO operations.
//! It handles terminal restore/init for operations that need the terminal.

mod bookmarks;
mod error;
mod git;
mod interactive;
mod operations;
mod revision;

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
    pub tree_refreshed: bool,
}

pub struct RunCtx<'a> {
    pub tree: &'a mut TreeState,
    pub diff_stats_cache: &'a mut std::collections::HashMap<String, DiffStats>,
    pub last_op: &'a mut Option<String>,
    pub result: RunResult,
}

impl<'a> RunCtx<'a> {
    pub fn new(
        tree: &'a mut TreeState,
        diff_stats_cache: &'a mut std::collections::HashMap<String, DiffStats>,
        last_op: &'a mut Option<String>,
    ) -> Self {
        Self {
            tree,
            diff_stats_cache,
            last_op,
            result: RunResult::default(),
        }
    }

    fn set_status(&mut self, text: impl Into<String>, kind: MessageKind) {
        self.result.status_message = Some((text.into(), kind));
    }

    fn success(&mut self, text: impl Into<String>) {
        self.set_status(text, MessageKind::Success);
    }

    fn warn(&mut self, text: impl Into<String>) {
        self.set_status(text, MessageKind::Warning);
    }

    fn error(&mut self, text: impl Into<String>) {
        self.set_status(text, MessageKind::Error);
    }

    fn refresh_tree(&mut self) {
        if let Err(error) = refresh::refresh_tree(self.tree, self.diff_stats_cache) {
            self.error(format!("Failed to refresh: {error}"));
        } else {
            self.result.tree_refreshed = true;
        }
    }
}

/// Execute a list of effects
pub fn run_effects(
    mut ctx: RunCtx<'_>,
    effects: Vec<Effect>,
    terminal: &mut DefaultTerminal,
) -> RunResult {
    for effect in effects {
        match effect {
            Effect::RefreshTree => ctx.refresh_tree(),
            Effect::SaveOperationForUndo => {
                if let Ok(op_id) = crate::cmd::jj_tui::commands::get_current_op_id() {
                    *ctx.last_op = Some(op_id);
                }
            }
            Effect::RunEdit { .. }
            | Effect::RunNew { .. }
            | Effect::RunCommit { .. }
            | Effect::RunAbandon { .. }
            | Effect::RunRebase { .. }
            | Effect::RunRebaseOntoTrunk { .. }
            | Effect::RunUndo
            | Effect::RunResolveDivergence { .. } => revision::handle(&mut ctx, effect),
            Effect::RunGitPush { .. }
            | Effect::RunGitPushMultiple { .. }
            | Effect::RunGitPushAll
            | Effect::RunStackSync
            | Effect::RunGitFetch
            | Effect::RunGitImport
            | Effect::RunGitExport
            | Effect::RunCreatePR { .. } => git::handle(&mut ctx, effect),
            Effect::RunBookmarkSet { .. }
            | Effect::RunBookmarkSetBackwards { .. }
            | Effect::RunBookmarkDelete { .. } => bookmarks::handle(&mut ctx, effect),
            Effect::RunInteractive(operation) => {
                interactive::handle(&mut ctx, terminal, operation);
            }
            Effect::SetStatus { text, kind } => ctx.set_status(text, kind),
            Effect::LoadConflictFiles => {}
        }
    }

    ctx.result
}
