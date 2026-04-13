mod bookmarks;
mod rebase;
mod stack_sync;

use crate::cmd::jj_tui::state::{MessageKind, RebaseType};

pub(super) fn run_bookmark_set(name: &str, rev: &str) -> (String, MessageKind) {
    bookmarks::run_bookmark_set(name, rev)
}

pub(super) fn run_bookmark_set_backwards(name: &str, rev: &str) -> (String, MessageKind) {
    bookmarks::run_bookmark_set_backwards(name, rev)
}

pub(super) fn run_rebase(
    source: &str,
    dest: &str,
    rebase_type: RebaseType,
    allow_branches: bool,
) -> (String, MessageKind) {
    rebase::run_rebase(source, dest, rebase_type, allow_branches)
}

pub(super) fn run_rebase_onto_trunk(
    source: &str,
    rebase_type: RebaseType,
) -> (String, MessageKind) {
    rebase::run_rebase_onto_trunk(source, rebase_type)
}

pub(super) fn run_stack_sync() -> (String, MessageKind) {
    stack_sync::run_stack_sync()
}
