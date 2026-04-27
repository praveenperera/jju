use super::super::App;
use crate::cmd::jj_tui::controller::{self, ControllerContext};
use crate::cmd::jj_tui::effect::Effect;
use crate::cmd::jj_tui::engine;
use crate::cmd::jj_tui::runner;
use crate::cmd::jj_tui::state::StatusMessage;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event;

impl App {
    pub(super) fn handle_key(
        &mut self,
        key: event::KeyEvent,
        viewport_height: usize,
        terminal: &mut DefaultTerminal,
    ) {
        self.expire_status_message();

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
        let old_mode = self.mode.clone();
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
                &self.repo_path,
            ),
            effects,
            terminal,
        );

        if needs_conflict_load {
            self.load_conflict_files();
        }

        self.transition_neighborhood_mode(&old_mode);

        if result.tree_refreshed {
            self.reset_row_data_loader();
        }

        if let Some((text, kind)) = result.status_message {
            if let Some(duration) = result.status_duration {
                self.status_message = Some(StatusMessage::with_duration(text, kind, duration));
            } else {
                self.set_status(&text, kind);
            }
        }
    }

    fn expire_status_message(&mut self) {
        if let Some(ref message) = self.status_message
            && message.is_expired()
        {
            self.status_message = None;
        }
    }
}
