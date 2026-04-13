use super::super::App;
use crate::cmd::jj_tui::{
    commands,
    state::{MessageKind, ModeState},
};

impl App {
    pub(super) fn load_conflict_files(&mut self) {
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
