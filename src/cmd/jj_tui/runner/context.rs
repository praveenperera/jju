use super::super::refresh;
use super::super::state::{DiffStats, MessageKind};
use super::super::tree::TreeState;
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

    pub(super) fn set_status(&mut self, text: impl Into<String>, kind: MessageKind) {
        self.result.status_message = Some((text.into(), kind));
    }

    pub(super) fn success(&mut self, text: impl Into<String>) {
        self.set_status(text, MessageKind::Success);
    }

    pub(super) fn warn(&mut self, text: impl Into<String>) {
        self.set_status(text, MessageKind::Warning);
    }

    pub(super) fn error(&mut self, text: impl Into<String>) {
        self.set_status(text, MessageKind::Error);
    }

    pub(super) fn refresh_tree(&mut self) {
        if let Err(error) = refresh::refresh_tree(self.tree, self.diff_stats_cache) {
            self.error(format!("Failed to refresh: {error}"));
        } else {
            self.result.tree_refreshed = true;
        }
    }
}
