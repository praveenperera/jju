mod fetch;
mod push;

use super::{Effect, RunCtx};

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
