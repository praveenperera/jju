//! Main application state and event loop for jj_tui
//!
//! This module contains the App struct and the main run loop.
//! Key handling is delegated to the controller, business logic to the engine,
//! and IO operations to the runner.

use super::controller::{self, ControllerContext};
use super::engine;
use super::handlers;
use super::runner;
use super::commands;
use super::effect::Effect;
use super::state::{DiffStats, MessageKind, ModeState, PendingOperation, PendingSquash, StatusMessage};
use super::tree::TreeState;
use super::ui;
use super::vm;
use crate::jj_lib_helpers::JjRepo;
use eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct App {
    pub tree: TreeState,
    pub mode: ModeState,
    pub should_quit: bool,
    pub split_view: bool,
    pub diff_stats_cache: std::collections::HashMap<String, DiffStats>,
    pub status_message: Option<StatusMessage>,
    pub pending_operation: Option<PendingOperation>,
    pub last_op: Option<String>,
    pub pending_key: Option<char>,
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) theme_set: ThemeSet,
}

impl App {
    pub fn new() -> Result<Self> {
        let jj_repo = JjRepo::load(None)?;
        let tree = TreeState::load(&jj_repo)?;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Ok(Self {
            tree,
            mode: ModeState::Normal,
            should_quit: false,
            split_view: false,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_operation: None,
            last_op: None,
            pending_key: None,
            syntax_set,
            theme_set,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let result = self.run_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            // handle pending operations that require terminal restoration
            match self.pending_operation.take() {
                Some(PendingOperation::EditDescription { rev }) => {
                    ratatui::restore();
                    let status = std::process::Command::new("jj")
                        .args(["describe", "-r", &rev])
                        .status();
                    *terminal = ratatui::init();
                    self.handle_edit_description_status(status);
                    continue;
                }
                Some(PendingOperation::Squash(squash)) => {
                    ratatui::restore();
                    let status = std::process::Command::new("jj")
                        .args(["squash", "-f", &squash.source_rev, "-t", &squash.target_rev])
                        .status();
                    *terminal = ratatui::init();
                    self.handle_squash_status(status, squash);
                    continue;
                }
                Some(PendingOperation::Resolve { file }) => {
                    ratatui::restore();
                    let status = commands::resolve::resolve_file(&file);
                    *terminal = ratatui::init();
                    self.handle_resolve_status(status, &file);
                    continue;
                }
                None => {}
            }

            let size = terminal.size()?;
            let viewport_height = size.height.saturating_sub(3) as usize;
            let viewport_width = size.width.saturating_sub(2) as usize;

            // fetch diff stats for expanded entry if needed
            self.ensure_expanded_stats();

            // build view models first to get accurate cursor height
            let vms = vm::build_tree_view(self, viewport_width);
            let cursor_height = vms.get(self.tree.cursor).map_or(1, |vm| vm.height);
            self.tree.update_scroll(viewport_height, cursor_height);

            terminal.draw(|frame| ui::render_with_vms(frame, self, &vms))?;

            if event::poll(std::time::Duration::from_millis(33))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key, viewport_height, terminal);
            }
        }

        Ok(())
    }

    fn handle_key(
        &mut self,
        key: event::KeyEvent,
        viewport_height: usize,
        terminal: &mut DefaultTerminal,
    ) {
        // clear expired status messages
        if let Some(ref msg) = self.status_message
            && msg.is_expired()
        {
            self.status_message = None;
        }

        // build controller context
        let ctx = ControllerContext {
            mode: &self.mode,
            pending_key: self.pending_key,
            viewport_height,
            has_focus: self.tree.is_focused(),
            has_selection: !self.tree.selected.is_empty(),
        };

        // map key to action
        let action = controller::handle_key(&ctx, key);

        // process action through engine
        let effects = engine::reduce(
            &mut self.tree,
            &mut self.mode,
            &mut self.should_quit,
            &mut self.split_view,
            &mut self.pending_key,
            &mut self.pending_operation,
            &self.syntax_set,
            &self.theme_set,
            action,
            viewport_height,
        );

        // check if we need to load conflict files before running effects
        let needs_conflict_load = effects.iter().any(|e| matches!(e, Effect::LoadConflictFiles));

        // execute effects
        let result = runner::run_effects(
            effects,
            &mut self.tree,
            &mut self.diff_stats_cache,
            &mut self.last_op,
            terminal,
        );

        // load conflict files if needed
        if needs_conflict_load {
            self.load_conflict_files();
        }

        // apply result
        if let Some((text, kind)) = result.status_message {
            self.set_status(&text, kind);
        }
    }

    fn load_conflict_files(&mut self) {
        let files = commands::list_conflict_files().unwrap_or_default();
        if let ModeState::Conflicts(ref mut state) = self.mode {
            state.files = files.clone();
            state.selected_index = 0;
        }

        if files.is_empty() {
            self.set_status("No conflicts in working copy", MessageKind::Success);
            self.mode = ModeState::Normal;
        }
    }

    fn handle_edit_description_status(&mut self, status: std::io::Result<std::process::ExitStatus>) {
        match status {
            Ok(s) if s.success() => {
                self.set_status("Description updated", MessageKind::Success);
                let _ = self.refresh_tree();
            }
            Ok(_) => self.set_status("Editor cancelled", MessageKind::Warning),
            Err(e) => self.set_status(&format!("Failed to launch editor: {e}"), MessageKind::Error),
        }
    }

    fn handle_squash_status(
        &mut self,
        status: std::io::Result<std::process::ExitStatus>,
        squash: PendingSquash,
    ) {
        match status {
            Ok(s) if s.success() => {
                self.last_op = Some(squash.op_before);
                let has_conflicts = self.check_conflicts();
                let _ = self.refresh_tree();

                if has_conflicts {
                    self.set_status(
                        "Squash created conflicts. Press u to undo",
                        MessageKind::Warning,
                    );
                } else {
                    self.set_status("Squash complete", MessageKind::Success);
                }
            }
            Ok(_) => self.set_status("Squash cancelled", MessageKind::Warning),
            Err(e) => self.set_status(&format!("Squash failed: {e}"), MessageKind::Error),
        }
    }

    fn handle_resolve_status(&mut self, status: eyre::Result<()>, file: &str) {
        match status {
            Ok(()) => {
                let has_conflicts = self.check_conflicts();
                let _ = self.refresh_tree();

                if has_conflicts {
                    self.set_status(
                        &format!("Resolved {}. More conflicts remain", file),
                        MessageKind::Warning,
                    );
                } else {
                    self.set_status("All conflicts resolved", MessageKind::Success);
                }
            }
            Err(e) => self.set_status(&format!("Resolve failed: {e}"), MessageKind::Error),
        }
    }

    fn set_status(&mut self, text: &str, kind: MessageKind) {
        self.status_message = Some(StatusMessage::new(text.to_string(), kind));
    }

    fn check_conflicts(&self) -> bool {
        super::commands::has_conflicts().unwrap_or(false)
    }

    fn refresh_tree(&mut self) -> Result<()> {
        // save current position to restore after refresh
        let current_change_id = self.tree.current_node().map(|n| n.change_id.clone());
        // save focus stack change_ids to restore after refresh
        let focus_stack_change_ids: Vec<String> = self
            .tree
            .focus_stack
            .iter()
            .filter_map(|&idx| self.tree.nodes.get(idx).map(|n| n.change_id.clone()))
            .collect();

        let jj_repo = JjRepo::load(None)?;
        self.tree = TreeState::load(&jj_repo)?;
        self.tree.clear_selection();
        self.diff_stats_cache.clear();

        // restore focus stack if the focused nodes still exist
        for change_id in focus_stack_change_ids {
            if let Some(node_idx) = self
                .tree
                .nodes
                .iter()
                .position(|n| n.change_id == change_id)
            {
                self.tree.focus_on(node_idx);
            }
        }

        // restore cursor to same change_id if it still exists
        if let Some(change_id) = current_change_id
            && let Some(idx) = self
                .tree
                .visible_entries
                .iter()
                .position(|e| self.tree.nodes[e.node_index].change_id == change_id)
        {
            self.tree.cursor = idx;
        }

        Ok(())
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

    pub fn ensure_expanded_stats(&mut self) {
        if let Some(entry) = self.tree.current_entry()
            && self.tree.is_expanded(self.tree.cursor)
        {
            let node = &self.tree.nodes[entry.node_index];
            let change_id = node.change_id.clone();
            let _ = self.get_diff_stats(&change_id);
        }
    }

    pub fn current_has_bookmark(&self) -> bool {
        self.tree
            .current_node()
            .map(|n| !n.bookmarks.is_empty())
            .unwrap_or(false)
    }

    /// Compute indices of entries that will move during rebase
    #[allow(dead_code)]
    pub fn compute_moving_indices(&self) -> ahash::HashSet<usize> {
        engine::compute_moving_indices(&self.tree, &self.mode)
    }
}
