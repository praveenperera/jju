use super::{Jj, JjCmd, split_hunk, stack_sync, tree};
use eyre::Result;

pub(super) fn run_with_flags(flags: Jj) -> Result<()> {
    let neighborhood = flags.neighborhood;
    match flags.subcommand {
        None => run_default(neighborhood),
        Some(JjCmd::StackSync { push, force }) => {
            stack_sync::StackSyncCommand::new(push, force).run()
        }
        Some(JjCmd::Tree { full, from }) => tree::TreeCommand::new(full, from).run(),
        Some(JjCmd::SplitHunk {
            message,
            revision,
            file,
            lines,
            hunks,
            pattern,
            preview,
            invert,
            dry_run,
        }) => split_hunk::SplitHunkCommand::new(split_hunk::SplitHunkOptions {
            message,
            revision,
            file_filter: file,
            lines,
            hunks,
            pattern,
            preview,
            invert,
            dry_run,
        })
        .run(),
    }
}

fn run_default(neighborhood: bool) -> Result<()> {
    if neighborhood {
        crate::cmd::jj_tui::run_with_options(crate::cmd::jj_tui::AppOptions {
            start_in_neighborhood: true,
        })
    } else {
        crate::cmd::jj_tui::run()
    }
}
