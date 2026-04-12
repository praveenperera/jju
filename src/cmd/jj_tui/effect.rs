//! Effect types for jj_tui
//!
//! Effects represent side effects - IO operations that need to be performed.
//! The engine produces effects, and the runner executes them.

use super::state::{MessageKind, RebaseType};
use jju_core::interactive::InteractiveOperation;

/// All possible side effects produced by the engine
#[derive(Debug, Clone)]
pub enum Effect {
    RefreshTree,

    // JJ commands
    RunEdit {
        rev: String,
    },
    RunNew {
        rev: String,
    },
    RunCommit {
        message: String,
    },
    RunAbandon {
        revset: String,
    },
    RunRebase {
        source: String,
        dest: String,
        rebase_type: RebaseType,
        allow_branches: bool,
    },
    RunRebaseOntoTrunk {
        source: String,
        rebase_type: RebaseType,
    },
    RunUndo,
    RunGitPush {
        bookmark: String,
    },
    RunGitPushMultiple {
        bookmarks: Vec<String>,
    },
    RunGitPushAll,
    RunGitFetch,
    RunStackSync,
    RunGitImport,
    RunGitExport,
    RunBookmarkSet {
        name: String,
        rev: String,
    },
    RunBookmarkSetBackwards {
        name: String,
        rev: String,
    },
    RunBookmarkDelete {
        name: String,
    },
    RunResolveDivergence {
        keep_commit_id: String,
        abandon_commit_ids: Vec<String>,
    },
    RunCreatePR {
        bookmark: String,
    },
    RunInteractive(InteractiveOperation),
    // UI
    SetStatus {
        text: String,
        kind: MessageKind,
    },

    // Operation tracking
    SaveOperationForUndo,

    // Conflicts
    LoadConflictFiles,
}
