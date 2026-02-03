//! State types for jj_tui
//!
//! This module contains all the state enums and structs used by the TUI.

use ratatui::style::Color;
use std::time::{Duration, Instant};

// Diff types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    FileHeader,
    Hunk,
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub fg: Color,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub spans: Vec<StyledSpan>,
    pub kind: DiffLineKind,
}

#[derive(Debug, Clone)]
pub struct DiffState {
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub rev: String,
}

// Mode state

/// Unified mode state - single source of truth for current mode and its associated state
#[derive(Debug, Clone)]
pub enum ModeState {
    Normal,
    Help,
    ViewingDiff(DiffState),
    Confirming(ConfirmState),
    Selecting,
    Rebasing(RebaseState),
    MovingBookmark(MovingBookmarkState),
    BookmarkInput(BookmarkInputState),
    #[allow(dead_code)]
    BookmarkSelect(BookmarkSelectState),
    BookmarkPicker(BookmarkPickerState),
    PushSelect(PushSelectState),
    Squashing(SquashState),
    Conflicts(ConflictsState),
}

impl ModeState {
    pub fn is_help(&self) -> bool {
        matches!(self, ModeState::Help)
    }
}

// Rebase types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseType {
    Single,          // -r: just this revision
    WithDescendants, // -s: revision + all descendants
}

impl std::fmt::Display for RebaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebaseType::Single => write!(f, "-r"),
            RebaseType::WithDescendants => write!(f, "-s"),
        }
    }
}

// Status message types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    #[allow(dead_code)]
    Info,
    Success,
    Warning,
    Error,
}

pub struct StatusMessage {
    pub text: String,
    pub kind: MessageKind,
    pub expires: Instant,
}

impl StatusMessage {
    pub fn new(text: String, kind: MessageKind) -> Self {
        Self {
            text,
            kind,
            expires: Instant::now() + Duration::from_secs(3),
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires
    }
}

// Confirmation state

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Abandon,
    RebaseOntoTrunk(RebaseType),
    MoveBookmarkBackwards {
        bookmark_name: String,
        dest_rev: String,
        op_before: String,
    },
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub action: ConfirmAction,
    pub message: String,
    pub revs: Vec<String>,
}

// Rebase state

#[derive(Debug, Clone)]
pub struct RebaseState {
    pub source_rev: String,
    pub rebase_type: RebaseType,
    pub dest_cursor: usize,
    pub allow_branches: bool,
    #[allow(dead_code)]
    pub op_before: String,
}

// Diff stats

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

// Bookmark states

#[derive(Debug, Clone)]
pub struct MovingBookmarkState {
    pub bookmark_name: String,
    pub dest_cursor: usize,
    pub op_before: String,
}

#[derive(Debug, Clone)]
pub struct BookmarkInputState {
    pub name: String,
    pub cursor: usize,
    pub target_rev: String,
    pub deleting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkSelectAction {
    Move,
    Delete,
}

#[derive(Debug, Clone)]
pub struct BookmarkSelectState {
    pub bookmarks: Vec<String>,
    pub selected_index: usize,
    pub target_rev: String,
    pub action: BookmarkSelectAction,
}

/// State for picking a bookmark from all bookmarks with type-to-filter
#[derive(Debug, Clone)]
pub struct BookmarkPickerState {
    pub all_bookmarks: Vec<String>,
    pub filter: String,
    pub filter_cursor: usize,
    pub selected_index: usize,
    pub target_rev: String,
    pub action: BookmarkSelectAction,
}

impl BookmarkPickerState {
    /// Get bookmarks that match the current filter
    pub fn filtered_bookmarks(&self) -> Vec<&String> {
        if self.filter.is_empty() {
            self.all_bookmarks.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.all_bookmarks
                .iter()
                .filter(|b| b.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }
}

/// State for multi-select bookmark push
#[derive(Debug, Clone)]
pub struct PushSelectState {
    pub all_bookmarks: Vec<String>,
    pub filter: String,
    pub filter_cursor: usize,
    pub cursor_index: usize,
    pub selected: ahash::HashSet<usize>, // indices into all_bookmarks (not filtered)
}

impl PushSelectState {
    /// Get bookmarks that match the current filter with their original indices
    pub fn filtered_bookmarks(&self) -> Vec<(usize, &str)> {
        if self.filter.is_empty() {
            self.all_bookmarks
                .iter()
                .enumerate()
                .map(|(i, s)| (i, s.as_str()))
                .collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.all_bookmarks
                .iter()
                .enumerate()
                .filter(|(_, b)| b.to_lowercase().contains(&filter_lower))
                .map(|(i, s)| (i, s.as_str()))
                .collect()
        }
    }

    /// Count selected bookmarks in the filtered view
    pub fn selected_filtered_count(&self) -> usize {
        self.filtered_bookmarks()
            .iter()
            .filter(|(idx, _)| self.selected.contains(idx))
            .count()
    }
}

// Squash state

#[derive(Debug, Clone)]
pub struct SquashState {
    pub source_rev: String,
    pub dest_cursor: usize,
    pub op_before: String,
}

pub struct PendingSquash {
    pub source_rev: String,
    pub target_rev: String,
    pub op_before: String,
}

/// Pending operations that require terminal restoration and external process execution
pub enum PendingOperation {
    EditDescription { rev: String },
    Squash(PendingSquash),
    Resolve { file: String },
}

// Conflicts state

#[derive(Debug, Clone, Default)]
pub struct ConflictsState {
    pub files: Vec<String>,
    pub selected_index: usize,
}
