use super::App;
use crate::cmd::jj_tui::controller::{self, ControllerContext};
use crate::cmd::jj_tui::effect::Effect;
use crate::cmd::jj_tui::engine;
use crate::cmd::jj_tui::runner;
use crate::cmd::jj_tui::state::{MessageKind, ModeState, StatusMessage};
use crate::cmd::jj_tui::{commands, ui, vm};
use eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

impl App {
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
        if let Some(ref message) = self.status_message
            && message.is_expired()
        {
            self.status_message = None;
        }

        let ctx = ControllerContext {
            mode: &self.mode,
            pending_key: self.pending_key,
            viewport_height,
            has_focus: self.tree.is_focused(),
            has_selection: !self.tree.view.selected.is_empty(),
            neighborhood_active: self.tree.is_neighborhood_mode(),
            has_neighborhood_history: self.tree.has_neighborhood_history(),
            can_enter_neighborhood_path: self.tree.current_entry_is_neighborhood_preview(),
        };
        let action = controller::handle_key(&ctx, key);
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
        let needs_conflict_load = effects
            .iter()
            .any(|effect| matches!(effect, Effect::LoadConflictFiles));
        let result = runner::run_effects(
            runner::RunCtx::new(
                &mut self.tree,
                &mut self.diff_stats_cache,
                &mut self.last_op,
            ),
            effects,
            terminal,
        );

        if needs_conflict_load {
            self.load_conflict_files();
        }

        if result.tree_refreshed {
            self.start_detail_hydration();
        }

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
}
