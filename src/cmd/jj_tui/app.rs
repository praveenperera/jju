//! Main application state and event loop for jj_tui
//!
//! This module contains the App struct and the main run loop.
//! Key handling is delegated to the controller, business logic to the engine,
//! and IO operations to the runner.

mod details;
mod runtime;
#[cfg(test)]
mod tests;

use super::handlers;
use super::keybindings;
use super::state::{DiffStats, MessageKind, ModeState, StatusMessage};
use super::tree::{TreeLoadScope, TreeState};
use crate::cmd::jj_tui::app::details::DetailHydrator;
use crate::jj_lib_helpers::JjRepo;
use eyre::Result;
use log::info;
use std::path::PathBuf;
use std::time::Instant;
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
    pub(crate) detail_hydrator: Option<DetailHydrator>,
    pub(crate) detail_generation: u64,
}

impl App {
    pub fn new(options: AppOptions) -> Result<Self> {
        let startup_started_at = Instant::now();
        let keybindings_warning = keybindings::initialize();
        let repo_path = std::env::current_dir()?;
        let jj_repo = JjRepo::load(Some(&repo_path))?;
        let load_scope = if options.start_in_neighborhood {
            TreeLoadScope::Neighborhood
        } else {
            TreeLoadScope::Stack
        };
        let mut tree = TreeState::load_with_scope(&jj_repo, "trunk()", load_scope)?;
        apply_startup_options(&mut tree, options);
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let mut app = Self {
            tree,
            mode: ModeState::Normal,
            should_quit: false,
            split_view: false,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: keybindings_warning.map(|warning| {
                StatusMessage::with_duration(
                    warning,
                    MessageKind::Warning,
                    keybindings::warning_duration(),
                )
            }),
            last_op: None,
            pending_key: None,
            syntax_set,
            theme_set,
            repo_path,
            detail_hydrator: None,
            detail_generation: 0,
        };
        app.start_detail_hydration();
        info!("Initialized jj_tui in {:?}", startup_started_at.elapsed());

        Ok(app)
    }

    fn set_status(&mut self, text: &str, kind: MessageKind) {
        self.status_message = Some(StatusMessage::new(text.to_string(), kind));
    }

    pub fn get_diff_stats(&mut self, change_id: &str) -> Option<&DiffStats> {
        if !self.diff_stats_cache.contains_key(change_id)
            && let Ok(stats) = self.fetch_diff_stats(change_id)
        {
            self.diff_stats_cache.insert(change_id.to_string(), stats);
        }
        self.diff_stats_cache.get(change_id)
    }

    fn fetch_diff_stats(&self, change_id: &str) -> Result<DiffStats> {
        let output = super::commands::diff::get_stats(change_id)?;
        Ok(handlers::diff::parse_diff_stats(&output))
    }

    pub fn current_has_bookmark(&self) -> bool {
        self.tree
            .current_node()
            .map(|n| !n.bookmarks.is_empty())
            .unwrap_or(false)
    }
}

pub(super) fn apply_startup_options(tree: &mut TreeState, options: AppOptions) {
    if options.start_in_neighborhood {
        tree.jump_to_working_copy();
        tree.enable_neighborhood();
    }
}
