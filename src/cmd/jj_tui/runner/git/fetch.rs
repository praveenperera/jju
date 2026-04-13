use super::GitRunner;
use crate::cmd::jj_tui::runner::operations;

impl GitRunner<'_, '_> {
    pub(super) fn run_stack_sync(&mut self) {
        let (text, kind) = operations::run_stack_sync();
        self.0.set_status(text, kind);
    }

    pub(super) fn run_fetch(&mut self) {
        match crate::cmd::jj_tui::commands::git::fetch() {
            Ok(_) => self.0.success("Git fetch complete"),
            Err(error) => self.0.error(format!("Git fetch failed: {error}")),
        }
    }

    pub(super) fn run_import(&mut self) {
        match crate::cmd::jj_tui::commands::git::import() {
            Ok(_) => self.0.success("Git import complete"),
            Err(error) => self.0.error(format!("Git import failed: {error}")),
        }
    }

    pub(super) fn run_export(&mut self) {
        match crate::cmd::jj_tui::commands::git::export() {
            Ok(_) => self.0.success("Git export complete"),
            Err(error) => self.0.error(format!("Git export failed: {error}")),
        }
    }
}
