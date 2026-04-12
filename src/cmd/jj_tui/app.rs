//! Main application state and event loop for jj_tui
//!
//! This module contains the App struct and the main run loop.
//! Key handling is delegated to the controller, business logic to the engine,
//! and IO operations to the runner.

use super::commands;
use super::controller::{self, ControllerContext};
use super::effect::Effect;
use super::engine;
use super::handlers;
use super::keybindings;
use super::runner;
use super::state::{DiffStats, MessageKind, ModeState, StatusMessage};
use super::tree::{TreeLoadScope, TreeState};
use super::ui;
use super::vm;
use crate::jj_lib_helpers::{CommitDetails, JjRepo};
use eyre::Result;
use log::info;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Instant;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Clone, Copy, Debug, Default)]
pub struct AppOptions {
    pub start_in_neighborhood: bool,
}

struct DetailHydrationUpdate {
    generation: u64,
    commit_id: String,
    details: CommitDetails,
}

pub(crate) struct DetailHydrator {
    generation: u64,
    receiver: Receiver<DetailHydrationUpdate>,
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

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let result = self.run_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            let size = terminal.size()?;
            let viewport_height = size.height.saturating_sub(3) as usize;
            let viewport_width = size.width.saturating_sub(2) as usize;

            self.apply_detail_updates();
            self.ensure_expanded_row_data();

            // build view models first to get accurate cursor height
            let vms = vm::build_tree_view(self, viewport_width);
            let cursor_vm = vms.get(self.tree.view.cursor);
            let cursor_height = cursor_vm.map_or(1, |vm| {
                vm.height + if vm.has_separator_before { 1 } else { 0 }
            });
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
            has_selection: !self.tree.view.selected.is_empty(),
        };

        // map key to action
        let action = controller::handle_key(&ctx, key);
        let old_mode = self.mode.clone();

        // process action through engine
        let effects = engine::reduce(
            engine::ReduceCtx::new(
                &mut self.tree,
                &mut self.mode,
                &mut self.should_quit,
                &mut self.split_view,
                &mut self.pending_key,
                engine::ReduceResources {
                    syntax_set: &self.syntax_set,
                    theme_set: &self.theme_set,
                },
            ),
            action,
        );

        // check if we need to load conflict files before running effects
        let needs_conflict_load = effects
            .iter()
            .any(|e| matches!(e, Effect::LoadConflictFiles));

        // execute effects
        let result = runner::run_effects(
            runner::RunCtx::new(
                &mut self.tree,
                &mut self.diff_stats_cache,
                &mut self.last_op,
            ),
            effects,
            terminal,
        );

        // load conflict files if needed
        if needs_conflict_load {
            self.load_conflict_files();
        }

        self.transition_neighborhood_mode(&old_mode);

        if result.tree_refreshed {
            self.start_detail_hydration();
        }

        // apply result
        if let Some((text, kind)) = result.status_message {
            if let Some(duration) = result.status_duration {
                self.status_message = Some(StatusMessage::with_duration(text, kind, duration));
            } else {
                self.set_status(&text, kind);
            }
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

    pub fn ensure_expanded_row_data(&mut self) {
        if let Some(entry) = self.tree.current_entry()
            && self.tree.is_expanded(self.tree.view.cursor)
        {
            let (commit_id, change_id, needs_details) = {
                let node = &self.tree.nodes()[entry.node_index];
                (
                    node.commit_id.clone(),
                    node.change_id.clone(),
                    !node.has_details(),
                )
            };

            if needs_details {
                self.load_node_details_sync(&commit_id);
            }
            let _ = self.get_diff_stats(&change_id);
        }
    }

    pub fn current_has_bookmark(&self) -> bool {
        self.tree
            .current_node()
            .map(|n| !n.bookmarks.is_empty())
            .unwrap_or(false)
    }

    fn transition_neighborhood_mode(&mut self, old_mode: &ModeState) {
        if !self.tree.is_neighborhood_mode() {
            return;
        }

        let old_is_normal = matches!(old_mode, ModeState::Normal);
        let new_is_normal = matches!(self.mode, ModeState::Normal);

        if old_is_normal && !new_is_normal {
            self.tree.freeze_neighborhood_anchor();
        } else if !old_is_normal && new_is_normal {
            self.tree.resume_neighborhood_follow_cursor();
        }
    }

    fn start_detail_hydration(&mut self) {
        let commit_ids = self.detail_hydration_order();
        if commit_ids.is_empty() {
            self.detail_hydrator = None;
            return;
        }

        self.detail_generation += 1;
        let generation = self.detail_generation;
        let repo_path = self.repo_path.clone();
        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            let started_at = Instant::now();
            let Ok(jj_repo) = JjRepo::load(Some(&repo_path)) else {
                return;
            };
            let Ok(hydrated_count) = jj_repo.with_short_prefix_index(|prefix_index| {
                let mut hydrated_count = 0;

                for commit_id in commit_ids {
                    let Ok(commit) = jj_repo.commit_by_id_hex(&commit_id) else {
                        continue;
                    };
                    let Ok(details) = jj_repo.commit_details_with_index(&commit, prefix_index)
                    else {
                        continue;
                    };

                    if sender
                        .send(DetailHydrationUpdate {
                            generation,
                            commit_id,
                            details,
                        })
                        .is_err()
                    {
                        break;
                    }

                    hydrated_count += 1;
                }

                Ok(hydrated_count)
            }) else {
                return;
            };

            info!(
                "Hydrated {} tree rows in {:?}",
                hydrated_count,
                started_at.elapsed()
            );
        });

        self.detail_hydrator = Some(DetailHydrator {
            generation,
            receiver,
        });
    }

    fn detail_hydration_order(&self) -> Vec<String> {
        let mut seen = ahash::HashSet::default();
        let mut commit_ids = Vec::new();

        if let Some(node) = self.tree.current_node()
            && seen.insert(node.commit_id.clone())
        {
            commit_ids.push(node.commit_id.clone());
        }

        for entry in self.tree.visible_entries() {
            let commit_id = self.tree.nodes()[entry.node_index].commit_id.clone();
            if seen.insert(commit_id.clone()) {
                commit_ids.push(commit_id);
            }
        }

        for node in self.tree.nodes() {
            if seen.insert(node.commit_id.clone()) {
                commit_ids.push(node.commit_id.clone());
            }
        }

        commit_ids
    }

    fn apply_detail_updates(&mut self) {
        let Some(hydrator) = &mut self.detail_hydrator else {
            return;
        };

        let generation = hydrator.generation;
        let mut updates = Vec::new();
        let mut disconnected = false;

        loop {
            match hydrator.receiver.try_recv() {
                Ok(update) => updates.push(update),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        for update in updates {
            if update.generation != generation {
                continue;
            }
            self.tree.hydrate_details(&update.commit_id, update.details);
        }

        if disconnected {
            self.detail_hydrator = None;
        }
    }

    fn load_node_details_sync(&mut self, commit_id: &str) {
        let Ok(jj_repo) = JjRepo::load(Some(&self.repo_path)) else {
            return;
        };
        let Ok(commit) = jj_repo.commit_by_id_hex(commit_id) else {
            return;
        };
        let Ok(details) = jj_repo.with_short_prefix_index(|prefix_index| {
            jj_repo.commit_details_with_index(&commit, prefix_index)
        }) else {
            return;
        };

        self.tree.hydrate_details(commit_id, details);
    }
}

fn apply_startup_options(tree: &mut TreeState, options: AppOptions) {
    if options.start_in_neighborhood {
        tree.jump_to_working_copy();
        tree.enable_neighborhood();
    }
}

#[cfg(test)]
mod tests {
    use super::{AppOptions, ModeState, apply_startup_options};
    use crate::cmd::jj_tui::app::App;
    use crate::cmd::jj_tui::test_support::{make_app_with_tree, make_node, make_tree};
    use crate::cmd::jj_tui::tree::TreeLoadScope;

    fn visible_ids(app: &App) -> Vec<String> {
        app.tree
            .visible_entries()
            .iter()
            .map(|entry| app.tree.nodes()[entry.node_index].change_id.clone())
            .collect()
    }

    #[test]
    fn startup_neighborhood_jumps_to_working_copy() {
        let mut nodes = vec![
            make_node("a", 0),
            make_node("b", 1),
            make_node("c", 2),
            make_node("d", 3),
        ];
        nodes[2].is_working_copy = true;
        let mut tree = make_tree(nodes);

        apply_startup_options(
            &mut tree,
            AppOptions {
                start_in_neighborhood: true,
            },
        );

        assert!(tree.is_neighborhood_mode());
        assert_eq!(tree.view.load_scope, TreeLoadScope::Neighborhood);
        assert_eq!(
            tree.current_node().map(|node| node.change_id.as_str()),
            Some("c")
        );
    }

    #[test]
    fn neighborhood_freezes_in_modal_modes_and_resumes_in_normal() {
        let nodes = vec![
            make_node("a", 0),
            make_node("b", 1),
            make_node("c", 2),
            make_node("d", 3),
            make_node("e", 4),
            make_node("f", 5),
            make_node("g", 6),
            make_node("h", 7),
        ];
        let tree = make_tree(nodes);
        let mut app = make_app_with_tree(tree);

        app.tree.view.cursor = 4;
        app.tree.enable_neighborhood();
        let initial_visible = visible_ids(&app);

        app.mode = ModeState::Selecting;
        app.transition_neighborhood_mode(&ModeState::Normal);
        assert!(!app.tree.is_neighborhood_following_cursor());

        app.tree.move_cursor_up();
        assert_eq!(visible_ids(&app), initial_visible);
        assert_eq!(
            app.tree.current_node().map(|node| node.change_id.as_str()),
            Some("d")
        );

        app.mode = ModeState::Normal;
        app.transition_neighborhood_mode(&ModeState::Selecting);
        assert!(app.tree.is_neighborhood_following_cursor());
        assert_eq!(visible_ids(&app), vec!["a", "b", "c", "d", "e", "f"]);
        assert_eq!(
            app.tree.current_node().map(|node| node.change_id.as_str()),
            Some("d")
        );
    }

    #[test]
    fn neighborhood_can_grow_and_shrink() {
        let nodes = vec![
            make_node("a", 0),
            make_node("b", 1),
            make_node("c", 2),
            make_node("d", 3),
            make_node("e", 4),
            make_node("f", 5),
            make_node("g", 6),
            make_node("h", 7),
            make_node("i", 8),
        ];
        let tree = make_tree(nodes);
        let mut app = make_app_with_tree(tree);

        app.tree.view.cursor = 4;
        app.tree.enable_neighborhood();
        assert_eq!(visible_ids(&app), vec!["a", "b", "c", "d", "e", "f", "g"]);

        assert!(app.tree.expand_neighborhood());
        assert_eq!(
            visible_ids(&app),
            vec!["a", "b", "c", "d", "e", "f", "g", "h", "i"]
        );

        assert!(app.tree.shrink_neighborhood());
        assert_eq!(visible_ids(&app), vec!["a", "b", "c", "d", "e", "f", "g"]);
    }
}
