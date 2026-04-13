use super::RevisionRunner;

impl RevisionRunner<'_, '_> {
    pub(super) fn run_edit(&mut self, rev: &str) {
        match crate::cmd::jj_tui::commands::revision::edit(rev) {
            Ok(_) => self.0.success(format!("Now editing {rev}")),
            Err(error) => self.0.error(format!("Edit failed: {error}")),
        }
    }

    pub(super) fn run_new(&mut self, rev: &str) {
        match crate::cmd::jj_tui::commands::revision::new(rev) {
            Ok(_) => self.0.success("Created new commit"),
            Err(error) => self.0.error(format!("Failed: {error}")),
        }
    }

    pub(super) fn run_commit(&mut self, message: &str) {
        match crate::cmd::jj_tui::commands::revision::commit(message) {
            Ok(_) => self.0.success("Changes committed"),
            Err(error) => self.0.error(format!("Commit failed: {error}")),
        }
    }
}
