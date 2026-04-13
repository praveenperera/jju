use super::super::App;
use crate::cmd::jj_tui::state::ModeState;

impl App {
    pub(super) fn transition_neighborhood_mode(&mut self, old_mode: &ModeState) {
        if !self.tree.is_neighborhood_mode() {
            return;
        }

        let old_is_normal = matches!(old_mode, ModeState::Normal);
        let new_is_normal = matches!(self.mode, ModeState::Normal);

        let _ = (old_is_normal, new_is_normal);
    }
}
