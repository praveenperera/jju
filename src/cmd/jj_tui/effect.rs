//! Effect types for jj_tui
//!
//! Effects represent side effects - IO operations that need to be performed.
//! The engine produces effects, and the runner executes them.

use super::state::{MessageKind, RebaseType};

/// All possible side effects produced by the engine
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Effect {
    // Lifecycle
    Quit,
    RefreshTree,

    // JJ commands
    RunEdit { rev: String },
    RunNew { rev: String },
    RunCommit { message: String },
    RunAbandon { revset: String },
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
    RunSquash {
        source: String,
        target: String,
    },
    RunUndo { op_id: String },
    RunGitPush { bookmark: String },
    RunGitImport,
    RunGitExport,
    RunBookmarkSet { name: String, rev: String },
    RunBookmarkSetBackwards { name: String, rev: String },
    RunBookmarkDelete { name: String },

    // Editor launch (requires terminal restore)
    LaunchDescriptionEditor { rev: String },
    LaunchSquashEditor {
        source: String,
        target: String,
        op_before: String,
    },

    // UI
    SetStatus { text: String, kind: MessageKind },

    // Operation tracking
    SaveOperationForUndo,
}
