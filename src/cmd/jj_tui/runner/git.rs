use super::{Effect, RunCtx};
use crate::cmd::jj_tui::runner::operations;
use crate::cmd::jj_tui::state::MessageKind;

pub(super) struct GitRunner<'a, 'b>(&'a mut RunCtx<'b>);

pub(super) fn handle(ctx: &mut RunCtx<'_>, effect: Effect) {
    let mut runner = GitRunner(ctx);

    match effect {
        Effect::RunGitPush { bookmark } => runner.run_push(&bookmark),
        Effect::RunGitPushMultiple { bookmarks } => runner.run_git_push_multiple(bookmarks),
        Effect::RunGitPushAll => runner.run_push_all(),
        Effect::RunStackSync => runner.run_stack_sync(),
        Effect::RunGitFetch => runner.run_fetch(),
        Effect::RunGitImport => runner.run_import(),
        Effect::RunGitExport => runner.run_export(),
        Effect::RunCreatePR { bookmark } => runner.run_create_pr(&bookmark),
        _ => unreachable!("unsupported git effect: {effect:?}"),
    }
}

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

    pub(super) fn run_create_pr(&mut self, bookmark: &str) {
        match crate::cmd::jj_tui::commands::git::push_and_pr(bookmark) {
            Ok(true) => self.0.success(format!("Pushed '{bookmark}' and opened PR")),
            Ok(false) => self
                .0
                .success(format!("Pushed '{bookmark}' and opened PR creation")),
            Err(error) => self.0.error(format!("PR failed: {error}")),
        }
    }
}
