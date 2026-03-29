use super::{Effect, MessageKind, RunCtx, operations};

pub(super) fn handle(ctx: &mut RunCtx<'_>, effect: Effect) {
    match effect {
        Effect::RunGitPush { bookmark } => {
            match crate::cmd::jj_tui::commands::git::push_bookmark(&bookmark) {
                Ok(_) => ctx.success(format!("Pushed bookmark '{bookmark}'")),
                Err(error) => ctx.error(format!("Push failed: {error}")),
            }
        }
        Effect::RunGitPushMultiple { bookmarks } => run_git_push_multiple(ctx, bookmarks),
        Effect::RunGitPushAll => match crate::cmd::jj_tui::commands::git::push_all() {
            Ok(_) => ctx.success("Pushed all bookmarks"),
            Err(error) => ctx.error(format!("Push all failed: {error}")),
        },
        Effect::RunStackSync => {
            let (text, kind) = operations::run_stack_sync();
            ctx.set_status(text, kind);
        }
        Effect::RunGitFetch => match crate::cmd::jj_tui::commands::git::fetch() {
            Ok(_) => ctx.success("Git fetch complete"),
            Err(error) => ctx.error(format!("Git fetch failed: {error}")),
        },
        Effect::RunGitImport => match crate::cmd::jj_tui::commands::git::import() {
            Ok(_) => ctx.success("Git import complete"),
            Err(error) => ctx.error(format!("Git import failed: {error}")),
        },
        Effect::RunGitExport => match crate::cmd::jj_tui::commands::git::export() {
            Ok(_) => ctx.success("Git export complete"),
            Err(error) => ctx.error(format!("Git export failed: {error}")),
        },
        Effect::RunCreatePR { bookmark } => {
            match crate::cmd::jj_tui::commands::git::push_and_pr(&bookmark) {
                Ok(true) => ctx.success(format!("Pushed '{bookmark}' and opened PR")),
                Ok(false) => ctx.success(format!("Pushed '{bookmark}' and opened PR creation")),
                Err(error) => ctx.error(format!("PR failed: {error}")),
            }
        }
        _ => unreachable!("unsupported git effect: {effect:?}"),
    }
}

fn run_git_push_multiple(ctx: &mut RunCtx<'_>, bookmarks: Vec<String>) {
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
            ctx.success(format!("Pushed bookmark '{}'", succeeded[0]));
        } else {
            ctx.success(format!("Pushed {} bookmarks", succeeded.len()));
        }
        return;
    }

    if succeeded.is_empty() {
        let first_err = &failed[0];
        if failed.len() == 1 {
            ctx.error(format!(
                "Push failed for '{}': {}",
                first_err.0, first_err.1
            ));
        } else {
            ctx.error(format!("Push failed for {} bookmarks", failed.len()));
        }
        return;
    }

    ctx.set_status(
        format!(
            "Pushed {} bookmarks, {} failed",
            succeeded.len(),
            failed.len()
        ),
        MessageKind::Warning,
    );
}
