mod split_hunk;
mod stack_sync;
mod tree;

use clap::{Parser, Subcommand};
use eyre::Result;
use log::debug;
use std::ffi::OsString;

#[derive(Debug, Clone, Parser)]
#[command(name = "jju", author, version, about, styles = crate::cli::get_styles())]
pub struct Jj {
    #[command(subcommand)]
    pub subcommand: Option<JjCmd>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum JjCmd {
    /// Sync the current stack with remote trunk (master/main/trunk)
    #[command(visible_alias = "ss")]
    StackSync {
        /// Push the first bookmark after syncing
        #[arg(short, long)]
        push: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Display the current stack as a tree
    #[command(visible_alias = "t")]
    Tree {
        /// Show all commits, including those without bookmarks
        #[arg(short, long)]
        full: bool,

        /// Base revision to start the tree from (default: trunk())
        #[arg(long)]
        from: Option<String>,
    },

    /// Split hunks from a commit non-interactively
    #[command(visible_alias = "sh")]
    SplitHunk {
        /// Commit message for the new commit (required unless --preview)
        #[arg(short, long)]
        message: Option<String>,

        /// Revision to split (default: @)
        #[arg(short, long, default_value = "@")]
        revision: String,

        /// File to select hunks from
        #[arg(long)]
        file: Option<String>,

        /// Line ranges to include (e.g., "10-20,30-40")
        #[arg(long)]
        lines: Option<String>,

        /// Hunk indices to include (e.g., "0,2,5")
        #[arg(long)]
        hunks: Option<String>,

        /// Regex pattern to match hunk content
        #[arg(long)]
        pattern: Option<String>,

        /// Preview hunks with indices (don't split)
        #[arg(long)]
        preview: bool,

        /// Exclude matched hunks instead of including them
        #[arg(long)]
        invert: bool,

        /// Show what would be committed without committing
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(args: &[OsString]) -> Result<()> {
    debug!("jj args: {args:?}");
    let flags = Jj::parse_from(args);
    run_with_flags(flags)
}

pub fn run_with_flags(flags: Jj) -> Result<()> {
    match flags.subcommand {
        None => crate::cmd::jj_tui::run(),
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
