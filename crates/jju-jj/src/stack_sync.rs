mod confirm;
mod discover;
mod execute;

use colored::Colorize;
use eyre::Result;

pub use discover::{
    cleanup_deleted_bookmarks, detect_trunk_branch, discover_plan, find_stack_roots,
    get_commit_description, sync_trunk_bookmark,
};
pub use execute::{execute_plan, rebase_root_onto_trunk};

#[derive(Debug, Clone, Copy)]
pub struct StackSyncCommand {
    push: bool,
    force: bool,
}

impl StackSyncCommand {
    pub fn new(push: bool, force: bool) -> Self {
        Self { push, force }
    }

    pub fn run(self) -> Result<()> {
        execute::run_command(self)
    }
}

fn print_aborted() {
    println!("{}", "Aborted".yellow());
}

fn print_complete() {
    println!("{}", "Stack sync complete".green());
}

fn should_continue(plan: &jju_core::stack_sync::StackSyncPlan, force: bool) -> Result<bool> {
    if force {
        return Ok(true);
    }

    confirm::confirm_plan(plan)
}
