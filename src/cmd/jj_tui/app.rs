//! Main application state and event loop for jj_tui
//!
//! This module contains the App struct and the main run loop.
//! Key handling is delegated to the controller, business logic to the engine,
//! and IO operations to the runner

mod replaceable_task;
mod row_data;
mod runtime;
mod startup;
#[cfg(test)]
mod tests;

use super::state::{DiffStats, ModeState, StatusMessage};
use super::tree::TreeState;
use crate::cmd::jj_tui::app::row_data::RowDataLoader;
use eyre::Result;
use std::path::PathBuf;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Clone, Copy, Debug, Default)]
pub struct AppOptions {
    pub start_in_neighborhood: bool,
}

pub struct App {
    pub tree: TreeState,
    pub mode: ModeState,
    pub should_quit: bool,
    pub split_view: bool,
    pub diff_stats_cache: std::collections::HashMap<String, DiffStats>,
    pub status_message: Option<StatusMessage>,
    pub last_op: Option<String>,
    pub pending_key: Option<char>,
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) theme_set: ThemeSet,
    pub(crate) repo_path: PathBuf,
    pub(crate) row_data_loader: RowDataLoader,
}

impl App {
    pub fn new(options: AppOptions) -> Result<Self> {
        startup::new_app(options)
    }

    pub fn current_has_bookmark(&self) -> bool {
        self.tree
            .current_node()
            .map(|node| !node.bookmarks.is_empty())
            .unwrap_or(false)
    }

    pub(super) fn set_status(&mut self, text: &str, kind: super::state::MessageKind) {
        self.status_message = Some(super::state::StatusMessage::new(text.to_string(), kind));
    }
}
