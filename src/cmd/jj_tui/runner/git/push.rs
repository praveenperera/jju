use super::GitRunner;
use crate::cmd::jj_tui::state::MessageKind;

impl GitRunner<'_, '_> {
    pub(super) fn run_push(&mut self, bookmark: &str) {
        match crate::cmd::jj_tui::commands::git::push_bookmark(bookmark) {
            Ok(_) => self.0.success(format!("Pushed bookmark '{bookmark}'")),
            Err(error) => self.0.error(format!("Push failed: {error}")),
        }
    }

    pub(super) fn run_push_all(&mut self) {
        match crate::cmd::jj_tui::commands::git::push_all() {
            Ok(_) => self.0.success("Pushed all bookmarks"),
            Err(error) => self.0.error(format!("Push all failed: {error}")),
        }
    }

    pub(super) fn run_git_push_multiple(&mut self, bookmarks: Vec<String>) {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();

        for bookmark in bookmarks {
            match crate::cmd::jj_tui::commands::git::push_bookmark(&bookmark) {
                Ok(_) => succeeded.push(bookmark),
                Err(error) => failed.push((bookmark, error.to_string())),
            }
        }

        if failed.is_empty() {
            if succeeded.len() == 1 {
                self.0
                    .success(format!("Pushed bookmark '{}'", succeeded[0]));
            } else {
                self.0
                    .success(format!("Pushed {} bookmarks", succeeded.len()));
            }
            return;
        }

        if succeeded.is_empty() {
            let first_err = &failed[0];
            if failed.len() == 1 {
                self.0.error(format!(
                    "Push failed for '{}': {}",
                    first_err.0, first_err.1
                ));
            } else {
                self.0
                    .error(format!("Push failed for {} bookmarks", failed.len()));
            }
            return;
        }

        self.0.set_status(
            format!(
                "Pushed {} bookmarks, {} failed",
                succeeded.len(),
                failed.len()
            ),
            MessageKind::Warning,
        );
    }
}
