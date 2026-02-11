//! Centralized keymap and keybinding rendering.
//!
//! This module is the single source of truth for:
//! - Mapping (ModeState, KeyEvent) -> Action
//! - Status bar / context help hints
//! - Prefix (chord) menus
//! - Help popup content

use std::sync::OnceLock;

use super::action::Action;
use super::state::{BookmarkSelectAction, ModeState, RebaseType};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModeId {
    Normal,
    Help,
    Diff,
    Confirm,
    Selecting,
    Rebase,
    Squash,
    MovingBookmark,
    BookmarkInput,
    BookmarkSelect,
    BookmarkPicker,
    PushSelect,
    Conflicts,
}

pub fn mode_id_from_state(mode: &ModeState) -> ModeId {
    match mode {
        ModeState::Normal => ModeId::Normal,
        ModeState::Help => ModeId::Help,
        ModeState::ViewingDiff(_) => ModeId::Diff,
        ModeState::Confirming(_) => ModeId::Confirm,
        ModeState::Selecting => ModeId::Selecting,
        ModeState::Rebasing(_) => ModeId::Rebase,
        ModeState::Squashing(_) => ModeId::Squash,
        ModeState::MovingBookmark(_) => ModeId::MovingBookmark,
        ModeState::BookmarkInput(_) => ModeId::BookmarkInput,
        ModeState::BookmarkSelect(_) => ModeId::BookmarkSelect,
        ModeState::BookmarkPicker(_) => ModeId::BookmarkPicker,
        ModeState::PushSelect(_) => ModeId::PushSelect,
        ModeState::Conflicts(_) => ModeId::Conflicts,
    }
}

// ============================================================================
// Runtime types (used for matching)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPattern {
    Exact {
        code: KeyCode,
        required_mods: KeyModifiers,
    },
    AnyChar,
}

impl KeyPattern {
    fn matches(&self, event: &KeyEvent) -> Option<MatchCapture> {
        match self {
            KeyPattern::Exact {
                code,
                required_mods,
            } => {
                if &event.code == code && event.modifiers.contains(*required_mods) {
                    Some(MatchCapture::None)
                } else {
                    None
                }
            }
            KeyPattern::AnyChar => match event.code {
                KeyCode::Char(c) => Some(MatchCapture::Char(c)),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchCapture {
    None,
    Char(char),
}

impl MatchCapture {
    fn char(self) -> Option<char> {
        match self {
            MatchCapture::None => None,
            MatchCapture::Char(c) => Some(c),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTemplate {
    Fixed(Action),
    PageUpHalfViewport,
    PageDownHalfViewport,
    CenterCursorViewport,
    BookmarkInputChar,
    BookmarkFilterChar,
    PushSelectFilterChar,
    NormalEscConditional,
}

impl ActionTemplate {
    fn build(&self, ctx: &ControllerContext<'_>, captured: Option<char>) -> Action {
        match self {
            ActionTemplate::Fixed(action) => action.clone(),
            ActionTemplate::PageUpHalfViewport => Action::PageUp(ctx.viewport_height / 2),
            ActionTemplate::PageDownHalfViewport => Action::PageDown(ctx.viewport_height / 2),
            ActionTemplate::CenterCursorViewport => Action::CenterCursor(ctx.viewport_height),
            ActionTemplate::BookmarkInputChar => Action::BookmarkInputChar(captured.unwrap_or(' ')),
            ActionTemplate::BookmarkFilterChar => {
                Action::BookmarkFilterChar(captured.unwrap_or(' '))
            }
            ActionTemplate::PushSelectFilterChar => {
                Action::PushSelectFilterChar(captured.unwrap_or(' '))
            }
            ActionTemplate::NormalEscConditional => {
                if ctx.has_focus {
                    Action::Unfocus
                } else if ctx.has_selection {
                    Action::ClearSelection
                } else {
                    Action::Noop
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    Primary,
    Alias,
}

pub struct Binding {
    pub mode: ModeId,
    pub pending_prefix: Option<char>,
    pub key: KeyPattern,
    pub action: ActionTemplate,
    pub display: DisplayKind,
    pub label: &'static str,
    pub help: Option<(&'static str, &'static str)>,
}

// ============================================================================
// Definition types (compact, config-file-ready)
// ============================================================================

#[derive(Clone, Copy)]
pub enum KeyDef {
    Char(char),
    Ctrl(char),
    Key(KeyCode),
    AnyChar,
}

impl KeyDef {
    const fn to_pattern(self) -> KeyPattern {
        match self {
            KeyDef::Char(c) => KeyPattern::Exact {
                code: KeyCode::Char(c),
                required_mods: KeyModifiers::NONE,
            },
            KeyDef::Ctrl(c) => KeyPattern::Exact {
                code: KeyCode::Char(c),
                required_mods: KeyModifiers::CONTROL,
            },
            KeyDef::Key(code) => KeyPattern::Exact {
                code,
                required_mods: KeyModifiers::NONE,
            },
            KeyDef::AnyChar => KeyPattern::AnyChar,
        }
    }
}

#[derive(Clone)]
pub struct BindingDef {
    pub mode: ModeId,
    pub prefix: Option<char>,
    pub key: KeyDef,
    pub action: ActionTemplate,
    pub display: DisplayKind,
    pub label: &'static str,
    pub help: Option<(&'static str, &'static str)>,
}

impl BindingDef {
    pub const fn new(
        mode: ModeId,
        key: KeyDef,
        action: ActionTemplate,
        label: &'static str,
    ) -> Self {
        Self {
            mode,
            key,
            action,
            label,
            prefix: None,
            display: DisplayKind::Primary,
            help: None,
        }
    }

    pub const fn prefix(mut self, prefix: char) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub const fn alias(mut self) -> Self {
        self.display = DisplayKind::Alias;
        self
    }

    pub const fn help(mut self, section: &'static str, desc: &'static str) -> Self {
        self.help = Some((section, desc));
        self
    }

    fn to_binding(&self) -> Binding {
        Binding {
            mode: self.mode,
            pending_prefix: self.prefix,
            key: self.key.to_pattern(),
            action: self.action.clone(),
            display: self.display,
            label: self.label,
            help: self.help,
        }
    }
}

// ============================================================================
// Binding definitions
// ============================================================================

use ActionTemplate::*;
use KeyDef::*;
use ModeId::*;

macro_rules! act {
    ($action:ident) => {
        Fixed(Action::$action)
    };
    ($action:ident($($arg:expr),*)) => {
        Fixed(Action::$action($($arg),*))
    };
}

pub const PREFIX_TITLES: &[(char, &str)] = &[('g', "git"), ('z', "nav"), ('b', "bookmark")];

static BINDING_DEFS: &[BindingDef] = &[
    // ========================================================================
    // Normal mode
    // ========================================================================
    BindingDef::new(Normal, Char('q'), act!(Quit), "quit").help("General", "Quit"),
    BindingDef::new(Normal, Ctrl('c'), act!(Quit), "quit").alias(),
    BindingDef::new(Normal, Char('Q'), act!(EnterSquashMode), "squash")
        .help("Rebase", "Squash into target"),
    BindingDef::new(Normal, Key(KeyCode::Esc), NormalEscConditional, "esc"),
    BindingDef::new(Normal, Char('?'), act!(EnterHelp), "help").help("General", "Toggle help"),
    // Navigation
    BindingDef::new(Normal, Char('j'), act!(MoveCursorDown), "down")
        .help("Navigation", "Move cursor down"),
    BindingDef::new(Normal, Key(KeyCode::Down), act!(MoveCursorDown), "down").alias(),
    BindingDef::new(Normal, Char('k'), act!(MoveCursorUp), "up")
        .help("Navigation", "Move cursor up"),
    BindingDef::new(Normal, Key(KeyCode::Up), act!(MoveCursorUp), "up").alias(),
    BindingDef::new(Normal, Char('@'), act!(JumpToWorkingCopy), "working-copy")
        .help("Navigation", "Jump to working copy"),
    BindingDef::new(Normal, Ctrl('u'), PageUpHalfViewport, "page-up").help("Navigation", "Page up"),
    BindingDef::new(Normal, Ctrl('d'), PageDownHalfViewport, "page-down")
        .help("Navigation", "Page down"),
    // Prefix keys
    BindingDef::new(Normal, Char('g'), act!(SetPendingKey('g')), "git"),
    BindingDef::new(Normal, Char('z'), act!(SetPendingKey('z')), "nav"),
    BindingDef::new(Normal, Char('b'), act!(SetPendingKey('b')), "bookmark"),
    // View
    BindingDef::new(Normal, Char('f'), act!(ToggleFullMode), "full")
        .help("View", "Toggle full mode"),
    BindingDef::new(Normal, Key(KeyCode::Enter), act!(ToggleFocus), "zoom")
        .help("Navigation", "Zoom in/out on node"),
    BindingDef::new(Normal, Key(KeyCode::Tab), act!(ToggleExpanded), "details")
        .help("View", "Toggle commit details"),
    BindingDef::new(Normal, Char(' '), act!(ToggleExpanded), "details").alias(),
    BindingDef::new(Normal, Char('\\'), act!(ToggleSplitView), "split")
        .help("View", "Toggle split view"),
    BindingDef::new(Normal, Char('R'), act!(RefreshTree), "refresh")
        .help("View", "Refresh tree"),
    BindingDef::new(Normal, Char('d'), act!(EnterDiffView), "diff").help("View", "View diff"),
    BindingDef::new(Normal, Char('D'), act!(EditDescription), "desc")
        .help("View", "Edit description"),
    // Edit operations
    BindingDef::new(Normal, Char('e'), act!(EditWorkingCopy), "edit")
        .help("Edit Operations", "Edit working copy (jj edit)"),
    BindingDef::new(Normal, Char('n'), act!(CreateNewCommit), "new")
        .help("Edit Operations", "New commit (jj new)"),
    BindingDef::new(Normal, Char('c'), act!(CommitWorkingCopy), "commit")
        .help("Edit Operations", "Commit changes (jj commit)"),
    // Selection
    BindingDef::new(Normal, Char('x'), act!(ToggleSelection), "toggle")
        .help("Selection", "Toggle selection"),
    BindingDef::new(Normal, Char('v'), act!(EnterSelecting), "select")
        .help("Selection", "Visual select mode"),
    BindingDef::new(Normal, Char('a'), act!(EnterConfirmAbandon), "abandon")
        .help("Selection", "Abandon selected"),
    // Rebase
    BindingDef::new(
        Normal,
        Char('r'),
        act!(EnterRebaseMode(RebaseType::Single)),
        "rebase-single",
    )
    .help("Rebase", "Rebase single (-r)"),
    BindingDef::new(
        Normal,
        Char('s'),
        act!(EnterRebaseMode(RebaseType::WithDescendants)),
        "rebase-desc",
    )
    .help("Rebase", "Rebase + descendants (-s)"),
    BindingDef::new(
        Normal,
        Char('t'),
        act!(EnterConfirmRebaseOntoTrunk(RebaseType::Single)),
        "trunk-single",
    )
    .help("Rebase", "Quick rebase onto trunk"),
    BindingDef::new(
        Normal,
        Char('T'),
        act!(EnterConfirmRebaseOntoTrunk(RebaseType::WithDescendants)),
        "trunk-desc",
    )
    .help("Rebase", "Quick rebase tree onto trunk"),
    BindingDef::new(Normal, Char('u'), act!(Undo), "undo").help("Rebase", "Undo last operation"),
    // Git
    BindingDef::new(Normal, Char('p'), act!(GitPush), "push")
        .help("Bookmarks & Git", "Push current bookmark"),
    BindingDef::new(Normal, Char('P'), act!(GitPushAll), "push-all"),
    BindingDef::new(Normal, Char('S'), act!(EnterConfirmStackSync), "stack-sync")
        .help("Bookmarks & Git", "Stack sync (fetch, rebase, clean up)"),
    BindingDef::new(Normal, Char('C'), act!(EnterConflicts), "conflicts")
        .help("Conflicts", "View conflicts panel"),
    // Normal chords: g (git)
    BindingDef::new(Normal, Char('f'), act!(GitFetch), "fetch")
        .prefix('g')
        .help("Bookmarks & Git", "Git fetch"),
    BindingDef::new(Normal, Char('i'), act!(GitImport), "import")
        .prefix('g')
        .help("Bookmarks & Git", "Git import"),
    BindingDef::new(Normal, Char('e'), act!(GitExport), "export")
        .prefix('g')
        .help("Bookmarks & Git", "Git export"),
    BindingDef::new(
        Normal,
        Char('r'),
        act!(ResolveDivergence),
        "resolve-divergence",
    )
    .prefix('g')
    .help("Bookmarks & Git", "Resolve divergence (keep local)"),
    BindingDef::new(Normal, Char('p'), act!(CreatePR), "create-pr")
        .prefix('g')
        .help("Bookmarks & Git", "Create/open PR from bookmark"),
    // Normal chords: z (nav)
    BindingDef::new(Normal, Char('t'), act!(MoveCursorTop), "top")
        .prefix('z')
        .help("Navigation", "Jump to top"),
    BindingDef::new(Normal, Char('b'), act!(MoveCursorBottom), "bottom")
        .prefix('z')
        .help("Navigation", "Jump to bottom"),
    BindingDef::new(Normal, Char('z'), CenterCursorViewport, "center")
        .prefix('z')
        .help("Navigation", "Center current line"),
    // Normal chords: b (bookmarks)
    BindingDef::new(Normal, Char('m'), act!(EnterMoveBookmarkMode), "move")
        .prefix('b')
        .help("Bookmarks & Git", "Move bookmark"),
    BindingDef::new(Normal, Char('s'), act!(EnterCreateBookmark), "set/new")
        .prefix('b')
        .help("Bookmarks & Git", "Set/create bookmark"),
    BindingDef::new(
        Normal,
        Char('d'),
        act!(EnterBookmarkPicker(BookmarkSelectAction::Delete)),
        "delete",
    )
    .prefix('b')
    .help("Bookmarks & Git", "Delete bookmark"),
    // ========================================================================
    // Help mode
    // ========================================================================
    BindingDef::new(Help, Char('q'), act!(ExitHelp), "close"),
    BindingDef::new(Help, Char('?'), act!(ExitHelp), "close").alias(),
    BindingDef::new(Help, Key(KeyCode::Esc), act!(ExitHelp), "close").alias(),
    // ========================================================================
    // Diff mode
    // ========================================================================
    BindingDef::new(Diff, Char('j'), act!(ScrollDiffDown(1)), "scroll-down"),
    BindingDef::new(
        Diff,
        Key(KeyCode::Down),
        act!(ScrollDiffDown(1)),
        "scroll-down",
    )
    .alias(),
    BindingDef::new(Diff, Char('k'), act!(ScrollDiffUp(1)), "scroll-up"),
    BindingDef::new(Diff, Key(KeyCode::Up), act!(ScrollDiffUp(1)), "scroll-up").alias(),
    BindingDef::new(Diff, Char('d'), act!(ScrollDiffDown(20)), "page-down"),
    BindingDef::new(Diff, Char('u'), act!(ScrollDiffUp(20)), "page-up"),
    BindingDef::new(Diff, Char('z'), act!(SetPendingKey('z')), "nav"),
    BindingDef::new(Diff, Char('q'), act!(ExitDiffView), "close"),
    BindingDef::new(Diff, Key(KeyCode::Esc), act!(ExitDiffView), "close").alias(),
    // Diff chords: z
    BindingDef::new(Diff, Char('t'), act!(ScrollDiffTop), "top").prefix('z'),
    BindingDef::new(Diff, Char('b'), act!(ScrollDiffBottom), "bottom").prefix('z'),
    // ========================================================================
    // Confirm mode
    // ========================================================================
    BindingDef::new(Confirm, Char('y'), act!(ConfirmYes), "yes"),
    BindingDef::new(Confirm, Key(KeyCode::Enter), act!(ConfirmYes), "yes").alias(),
    BindingDef::new(Confirm, Char('n'), act!(ConfirmNo), "no"),
    BindingDef::new(Confirm, Key(KeyCode::Esc), act!(ConfirmNo), "no").alias(),
    // ========================================================================
    // Selecting mode
    // ========================================================================
    BindingDef::new(Selecting, Char('j'), act!(MoveCursorDown), "down"),
    BindingDef::new(Selecting, Key(KeyCode::Down), act!(MoveCursorDown), "down").alias(),
    BindingDef::new(Selecting, Char('k'), act!(MoveCursorUp), "up"),
    BindingDef::new(Selecting, Key(KeyCode::Up), act!(MoveCursorUp), "up").alias(),
    BindingDef::new(Selecting, Key(KeyCode::Esc), act!(ExitSelecting), "exit"),
    BindingDef::new(Selecting, Char('a'), act!(EnterConfirmAbandon), "abandon"),
    // ========================================================================
    // Rebase mode
    // ========================================================================
    BindingDef::new(Rebase, Char('j'), act!(MoveRebaseDestDown), "dest-down"),
    BindingDef::new(
        Rebase,
        Key(KeyCode::Down),
        act!(MoveRebaseDestDown),
        "dest-down",
    )
    .alias(),
    BindingDef::new(Rebase, Char('k'), act!(MoveRebaseDestUp), "dest-up"),
    BindingDef::new(Rebase, Key(KeyCode::Up), act!(MoveRebaseDestUp), "dest-up").alias(),
    BindingDef::new(Rebase, Char('b'), act!(ToggleRebaseBranches), "branches"),
    BindingDef::new(Rebase, Key(KeyCode::Enter), act!(ExecuteRebase), "run"),
    BindingDef::new(Rebase, Key(KeyCode::Esc), act!(ExitRebaseMode), "cancel"),
    // ========================================================================
    // Squash mode
    // ========================================================================
    BindingDef::new(Squash, Char('j'), act!(MoveSquashDestDown), "dest-down"),
    BindingDef::new(
        Squash,
        Key(KeyCode::Down),
        act!(MoveSquashDestDown),
        "dest-down",
    )
    .alias(),
    BindingDef::new(Squash, Char('k'), act!(MoveSquashDestUp), "dest-up"),
    BindingDef::new(Squash, Key(KeyCode::Up), act!(MoveSquashDestUp), "dest-up").alias(),
    BindingDef::new(Squash, Key(KeyCode::Enter), act!(ExecuteSquash), "run"),
    BindingDef::new(Squash, Key(KeyCode::Esc), act!(ExitSquashMode), "cancel"),
    // ========================================================================
    // MovingBookmark mode
    // ========================================================================
    BindingDef::new(
        MovingBookmark,
        Char('j'),
        act!(MoveBookmarkDestDown),
        "dest-down",
    ),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Down),
        act!(MoveBookmarkDestDown),
        "dest-down",
    )
    .alias(),
    BindingDef::new(
        MovingBookmark,
        Char('k'),
        act!(MoveBookmarkDestUp),
        "dest-up",
    ),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Up),
        act!(MoveBookmarkDestUp),
        "dest-up",
    )
    .alias(),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Enter),
        act!(ExecuteBookmarkMove),
        "run",
    ),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    // ========================================================================
    // BookmarkInput mode
    // ========================================================================
    BindingDef::new(
        BookmarkInput,
        Key(KeyCode::Enter),
        act!(ConfirmBookmarkInput),
        "confirm",
    ),
    BindingDef::new(
        BookmarkInput,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    BindingDef::new(
        BookmarkInput,
        Key(KeyCode::Backspace),
        act!(BookmarkInputBackspace),
        "backspace",
    )
    .alias(),
    BindingDef::new(
        BookmarkInput,
        Key(KeyCode::Delete),
        act!(BookmarkInputDelete),
        "delete",
    )
    .alias(),
    BindingDef::new(
        BookmarkInput,
        Key(KeyCode::Left),
        act!(BookmarkInputCursorLeft),
        "left",
    )
    .alias(),
    BindingDef::new(
        BookmarkInput,
        Key(KeyCode::Right),
        act!(BookmarkInputCursorRight),
        "right",
    )
    .alias(),
    BindingDef::new(BookmarkInput, AnyChar, BookmarkInputChar, "type"),
    // ========================================================================
    // BookmarkSelect mode
    // ========================================================================
    BindingDef::new(BookmarkSelect, Char('j'), act!(SelectBookmarkDown), "down"),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Down),
        act!(SelectBookmarkDown),
        "down",
    )
    .alias(),
    BindingDef::new(BookmarkSelect, Char('k'), act!(SelectBookmarkUp), "up"),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Up),
        act!(SelectBookmarkUp),
        "up",
    )
    .alias(),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Enter),
        act!(ConfirmBookmarkSelect),
        "select",
    ),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    // ========================================================================
    // BookmarkPicker mode
    // ========================================================================
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Enter),
        act!(ConfirmBookmarkPicker),
        "confirm",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Down),
        act!(BookmarkPickerDown),
        "down",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Up),
        act!(BookmarkPickerUp),
        "up",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Backspace),
        act!(BookmarkFilterBackspace),
        "backspace",
    )
    .alias(),
    BindingDef::new(BookmarkPicker, AnyChar, BookmarkFilterChar, "type"),
    // ========================================================================
    // PushSelect mode
    // ========================================================================
    BindingDef::new(
        PushSelect,
        Key(KeyCode::Esc),
        act!(ExitPushSelect),
        "cancel",
    ),
    BindingDef::new(
        PushSelect,
        Key(KeyCode::Enter),
        act!(PushSelectConfirm),
        "push",
    ),
    BindingDef::new(PushSelect, Key(KeyCode::Down), act!(PushSelectDown), "down"),
    BindingDef::new(PushSelect, Key(KeyCode::Up), act!(PushSelectUp), "up"),
    BindingDef::new(PushSelect, Char(' '), act!(PushSelectToggle), "toggle"),
    BindingDef::new(PushSelect, Char('a'), act!(PushSelectAll), "all"),
    BindingDef::new(PushSelect, Char('n'), act!(PushSelectNone), "none"),
    BindingDef::new(
        PushSelect,
        Key(KeyCode::Backspace),
        act!(PushSelectFilterBackspace),
        "backspace",
    )
    .alias(),
    BindingDef::new(PushSelect, AnyChar, PushSelectFilterChar, "type"),
    // ========================================================================
    // Conflicts mode
    // ========================================================================
    BindingDef::new(Conflicts, Char('j'), act!(ConflictsDown), "down"),
    BindingDef::new(Conflicts, Key(KeyCode::Down), act!(ConflictsDown), "down").alias(),
    BindingDef::new(Conflicts, Char('k'), act!(ConflictsUp), "up"),
    BindingDef::new(Conflicts, Key(KeyCode::Up), act!(ConflictsUp), "up").alias(),
    BindingDef::new(Conflicts, Key(KeyCode::Enter), act!(ConflictsJump), "jump"),
    BindingDef::new(
        Conflicts,
        Char('R'),
        act!(StartResolveFromConflicts),
        "resolve",
    ),
    BindingDef::new(Conflicts, Char('q'), act!(ExitConflicts), "exit"),
    BindingDef::new(Conflicts, Key(KeyCode::Esc), act!(ExitConflicts), "exit").alias(),
];

// ============================================================================
// Runtime binding access
// ============================================================================

static BINDINGS: OnceLock<Vec<Binding>> = OnceLock::new();

fn bindings() -> &'static [Binding] {
    BINDINGS.get_or_init(|| BINDING_DEFS.iter().map(|d| d.to_binding()).collect())
}

pub fn prefix_title(prefix: char) -> Option<&'static str> {
    PREFIX_TITLES
        .iter()
        .find(|(p, _)| *p == prefix)
        .map(|(_, t)| *t)
}

pub fn is_known_prefix(prefix: char) -> bool {
    prefix_title(prefix).is_some()
}

// ============================================================================
// Key handling
// ============================================================================

pub struct ControllerContext<'a> {
    pub mode: &'a ModeState,
    pub pending_key: Option<char>,
    pub viewport_height: usize,
    pub has_focus: bool,
    pub has_selection: bool,
}

pub fn handle_key(ctx: &ControllerContext<'_>, key: KeyEvent) -> Action {
    let mode = mode_id_from_state(ctx.mode);

    if let Some(pending) = ctx.pending_key {
        return match_binding(ctx, mode, Some(pending), &key).unwrap_or(Action::ClearPendingKey);
    }

    match_binding(ctx, mode, None, &key).unwrap_or(Action::Noop)
}

fn match_binding(
    ctx: &ControllerContext<'_>,
    mode: ModeId,
    pending_prefix: Option<char>,
    key: &KeyEvent,
) -> Option<Action> {
    for binding in bindings() {
        if binding.mode != mode || binding.pending_prefix != pending_prefix {
            continue;
        }
        if let Some(captured) = binding.key.matches(key) {
            return Some(binding.action.build(ctx, captured.char()));
        }
    }
    None
}

// ============================================================================
// Prefix menu
// ============================================================================

#[derive(Debug, Clone)]
pub struct PrefixMenuView {
    pub title: &'static str,
    pub items: Vec<(String, &'static str)>,
}

pub fn prefix_menu(mode: ModeId, pending: char) -> Option<PrefixMenuView> {
    let title = prefix_title(pending)?;
    let mut items = Vec::new();
    for binding in bindings() {
        if binding.mode != mode || binding.pending_prefix != Some(pending) {
            continue;
        }
        if binding.display != DisplayKind::Primary {
            continue;
        }
        items.push((
            format_binding_key(binding, KeyFormat::SecondKeyOnly),
            binding.label,
        ));
    }
    Some(PrefixMenuView { title, items })
}

// ============================================================================
// Key formatting
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum KeyFormat {
    Space,
    Concat,
    SecondKeyOnly,
}

pub fn format_binding_key(binding: &Binding, fmt: KeyFormat) -> String {
    let key = display_key_pattern(&binding.key);
    match (binding.pending_prefix, fmt) {
        (Some(_prefix), KeyFormat::SecondKeyOnly) => key,
        (Some(prefix), KeyFormat::Space) => format!("{prefix} {key}"),
        (Some(prefix), KeyFormat::Concat) => format!("{prefix}{key}"),
        (None, _) => key,
    }
}

pub fn display_keys_for_label(
    mode: ModeId,
    pending_prefix: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_format: KeyFormat,
) -> Vec<String> {
    keys_for_label(mode, pending_prefix, label, include_aliases, chord_format)
}

pub fn display_keys_joined(
    mode: ModeId,
    pending_prefix: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_format: KeyFormat,
    sep: &str,
) -> String {
    join_keys(
        &display_keys_for_label(mode, pending_prefix, label, include_aliases, chord_format),
        sep,
    )
}

fn display_key_pattern(key: &KeyPattern) -> String {
    match key {
        KeyPattern::Exact {
            code,
            required_mods,
        } => display_key_code(*code, *required_mods),
        KeyPattern::AnyChar => "type".to_string(),
    }
}

fn display_key_code(code: KeyCode, mods: KeyModifiers) -> String {
    let base = match code {
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::Char(c) => c.to_string(),
        other => format!("{other:?}"),
    };

    if mods.contains(KeyModifiers::CONTROL) {
        format!("Ctrl+{base}")
    } else {
        base
    }
}

fn find_bindings(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
) -> impl Iterator<Item = &'static Binding> {
    bindings()
        .iter()
        .filter(move |b| b.mode == mode && b.pending_prefix == pending && b.label == label)
}

fn first_key(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
    kind: DisplayKind,
) -> Option<String> {
    find_bindings(mode, pending, label)
        .find(|b| b.display == kind)
        .map(|b| format_binding_key(b, KeyFormat::Space))
}

fn keys_for_label(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_fmt: KeyFormat,
) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for binding in find_bindings(mode, pending, label) {
        if !include_aliases && binding.display != DisplayKind::Primary {
            continue;
        }
        let k = match (binding.pending_prefix, chord_fmt) {
            (Some(_), KeyFormat::SecondKeyOnly) => {
                format_binding_key(binding, KeyFormat::SecondKeyOnly)
            }
            (Some(_), KeyFormat::Concat) => format_binding_key(binding, KeyFormat::Concat),
            (Some(_), KeyFormat::Space) => format_binding_key(binding, KeyFormat::Space),
            (None, _) => display_key_pattern(&binding.key),
        };
        keys.push(k);
    }
    keys
}

fn join_keys(keys: &[String], sep: &str) -> String {
    keys.join(sep)
}

// ============================================================================
// Status bar hints
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct StatusHintContext {
    pub mode: ModeId,
    pub has_selection: bool,
    pub has_focus: bool,
    pub current_has_bookmark: bool,
    pub rebase_allow_branches: Option<bool>,
}

pub fn status_bar_hints(ctx: &StatusHintContext) -> String {
    match ctx.mode {
        ModeId::Normal => {
            if ctx.has_selection {
                join_segments(&[
                    kv(ModeId::Normal, None, "abandon", "abandon"),
                    kv(ModeId::Normal, None, "toggle", "toggle"),
                    kv(ModeId::Normal, None, "esc", "clear"),
                ])
            } else if ctx.has_focus {
                join_segments(&[
                    kv(ModeId::Normal, None, "zoom", "unfocus"),
                    kv(ModeId::Normal, None, "full", "toggle-full"),
                    kv(ModeId::Normal, None, "help", "help"),
                    kv(ModeId::Normal, None, "quit", "quit"),
                ])
            } else if ctx.current_has_bookmark {
                join_segments(&[
                    kv(ModeId::Normal, None, "push", "push"),
                    kv(ModeId::Normal, None, "bookmark", "bookmark"),
                    kv(ModeId::Normal, None, "rebase-single", "rebase"),
                    kv(ModeId::Normal, None, "help", "help"),
                    kv(ModeId::Normal, None, "quit", "quit"),
                ])
            } else {
                join_segments(&[
                    format!(
                        "{}/{}:rebase",
                        key_for_hint(ModeId::Normal, None, "rebase-single"),
                        key_for_hint(ModeId::Normal, None, "rebase-desc")
                    ),
                    kv(ModeId::Normal, None, "trunk-single", "trunk"),
                    kv(ModeId::Normal, None, "desc", "desc"),
                    kv(ModeId::Normal, None, "bookmark", "bookmark"),
                    kv(ModeId::Normal, None, "git", "git"),
                    kv(ModeId::Normal, None, "nav", "nav"),
                    kv(ModeId::Normal, None, "help", "help"),
                    kv(ModeId::Normal, None, "quit", "quit"),
                ])
            }
        }
        ModeId::Help => {
            let keys = keys_for_label(ModeId::Help, None, "close", true, KeyFormat::Space);
            format!("{}:close", join_keys(&keys, "/"))
        }
        ModeId::Diff => join_segments(&[
            format!(
                "{}/{}:scroll",
                key_for_hint(ModeId::Diff, None, "scroll-down"),
                key_for_hint(ModeId::Diff, None, "scroll-up")
            ),
            format!(
                "{}/{}:page",
                key_for_hint(ModeId::Diff, None, "page-down"),
                key_for_hint(ModeId::Diff, None, "page-up")
            ),
            format!(
                "{}/{}:top/bottom",
                key_for_hint_chord(ModeId::Diff, 'z', "top"),
                key_for_hint_chord(ModeId::Diff, 'z', "bottom")
            ),
            {
                let keys = keys_for_label(ModeId::Diff, None, "close", true, KeyFormat::Space);
                format!("{}:close", join_keys(&keys, "/"))
            },
        ]),
        ModeId::Confirm => join_segments(&[
            {
                let keys = keys_for_label(ModeId::Confirm, None, "yes", true, KeyFormat::Space);
                format!("{}:yes", join_keys(&keys, "/"))
            },
            {
                let keys = keys_for_label(ModeId::Confirm, None, "no", true, KeyFormat::Space);
                format!("{}:no", join_keys(&keys, "/"))
            },
        ]),
        ModeId::Selecting => join_segments(&[
            format!(
                "{}/{}:extend",
                key_for_hint(ModeId::Selecting, None, "down"),
                key_for_hint(ModeId::Selecting, None, "up")
            ),
            kv(ModeId::Selecting, None, "abandon", "abandon"),
            kv(ModeId::Selecting, None, "exit", "exit"),
        ]),
        ModeId::Rebase => {
            let b_label = if ctx.rebase_allow_branches.unwrap_or(false) {
                "inline"
            } else {
                "branch"
            };
            join_segments(&[
                format!(
                    "{}/{}:dest",
                    key_for_hint(ModeId::Rebase, None, "dest-down"),
                    key_for_hint(ModeId::Rebase, None, "dest-up")
                ),
                kv(ModeId::Rebase, None, "branches", b_label),
                kv(ModeId::Rebase, None, "run", "run"),
                kv(ModeId::Rebase, None, "cancel", "cancel"),
            ])
        }
        ModeId::Squash => join_segments(&[
            format!(
                "{}/{}:dest",
                key_for_hint(ModeId::Squash, None, "dest-down"),
                key_for_hint(ModeId::Squash, None, "dest-up")
            ),
            kv(ModeId::Squash, None, "run", "run"),
            kv(ModeId::Squash, None, "cancel", "cancel"),
        ]),
        ModeId::MovingBookmark => join_segments(&[
            format!(
                "{}/{}:dest",
                key_for_hint(ModeId::MovingBookmark, None, "dest-down"),
                key_for_hint(ModeId::MovingBookmark, None, "dest-up")
            ),
            kv(ModeId::MovingBookmark, None, "run", "run"),
            kv(ModeId::MovingBookmark, None, "cancel", "cancel"),
        ]),
        ModeId::BookmarkInput => join_segments(&[
            kv(ModeId::BookmarkInput, None, "confirm", "confirm"),
            kv(ModeId::BookmarkInput, None, "cancel", "cancel"),
        ]),
        ModeId::BookmarkSelect => join_segments(&[
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::BookmarkSelect, None, "down"),
                key_for_hint(ModeId::BookmarkSelect, None, "up")
            ),
            kv(ModeId::BookmarkSelect, None, "select", "select"),
            kv(ModeId::BookmarkSelect, None, "cancel", "cancel"),
        ]),
        ModeId::BookmarkPicker => join_segments(&[
            "type:filter".to_string(),
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::BookmarkPicker, None, "up"),
                key_for_hint(ModeId::BookmarkPicker, None, "down")
            ),
            kv(ModeId::BookmarkPicker, None, "confirm", "select"),
            kv(ModeId::BookmarkPicker, None, "cancel", "cancel"),
        ]),
        ModeId::PushSelect => join_segments(&[
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::PushSelect, None, "up"),
                key_for_hint(ModeId::PushSelect, None, "down")
            ),
            kv(ModeId::PushSelect, None, "toggle", "toggle"),
            kv(ModeId::PushSelect, None, "all", "all"),
            kv(ModeId::PushSelect, None, "none", "none"),
            kv(ModeId::PushSelect, None, "push", "push"),
            kv(ModeId::PushSelect, None, "cancel", "cancel"),
        ]),
        ModeId::Conflicts => join_segments(&[
            format!(
                "{}/{}:nav",
                key_for_hint(ModeId::Conflicts, None, "down"),
                key_for_hint(ModeId::Conflicts, None, "up")
            ),
            kv(ModeId::Conflicts, None, "resolve", "resolve"),
            {
                let keys = keys_for_label(ModeId::Conflicts, None, "exit", true, KeyFormat::Space);
                format!("{}:exit", join_keys(&keys, "/"))
            },
        ]),
    }
}

fn key_for_hint(mode: ModeId, pending: Option<char>, label: &str) -> String {
    first_key(mode, pending, label, DisplayKind::Primary)
        .unwrap_or_else(|| "?".to_string())
        .split_whitespace()
        .last()
        .unwrap_or("?")
        .to_string()
}

fn key_for_hint_chord(mode: ModeId, prefix: char, label: &str) -> String {
    keys_for_label(mode, Some(prefix), label, false, KeyFormat::Concat)
        .into_iter()
        .next()
        .unwrap_or_else(|| "?".to_string())
}

fn kv(mode: ModeId, pending: Option<char>, label: &str, value: &str) -> String {
    format!("{}:{value}", key_for_hint(mode, pending, label))
}

fn join_segments(segments: &[String]) -> String {
    segments.join("  ")
}

// ============================================================================
// Help view (derived from bindings)
// ============================================================================

const HELP_SECTION_ORDER: &[&str] = &[
    "Navigation",
    "View",
    "Edit Operations",
    "Selection",
    "Rebase",
    "Bookmarks & Git",
    "Conflicts",
    "General",
];

#[derive(Debug, Clone)]
pub struct HelpItemView {
    pub keys: String,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct HelpSectionView {
    pub title: &'static str,
    pub items: Vec<HelpItemView>,
}

pub fn build_help_view() -> Vec<HelpSectionView> {
    use std::collections::HashMap;

    let mut sections: HashMap<&'static str, Vec<HelpItemView>> = HashMap::new();

    for binding in bindings() {
        let Some((section, description)) = binding.help else {
            continue;
        };

        let keys = if binding.pending_prefix.is_some() {
            format_binding_key(binding, KeyFormat::Space)
        } else {
            display_key_pattern(&binding.key)
        };

        sections
            .entry(section)
            .or_default()
            .push(HelpItemView { keys, description });
    }

    // Also add aliases for items that should show multiple keys
    let alias_items = [
        (Normal, None, "down", "Navigation", "Move cursor down"),
        (Normal, None, "up", "Navigation", "Move cursor up"),
        (Normal, None, "details", "View", "Toggle commit details"),
    ];

    for (mode, prefix, label, section, _desc) in alias_items {
        let keys = keys_for_label(mode, prefix, label, true, KeyFormat::Space);
        if keys.len() > 1 {
            let joined = join_keys(&keys, "/");
            if let Some(items) = sections.get_mut(section) {
                for item in items.iter_mut() {
                    if item.keys == keys[0] {
                        item.keys = joined;
                        break;
                    }
                }
            }
        }
    }

    // Build final view in order
    let mut out = Vec::new();
    for &title in HELP_SECTION_ORDER {
        if let Some(items) = sections.remove(title) {
            out.push(HelpSectionView { title, items });
        }
    }

    // Add any remaining sections not in the order list
    for (title, items) in sections {
        out.push(HelpSectionView { title, items });
    }

    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::state::{
        BookmarkInputState, BookmarkPickerState, ConfirmAction, ConfirmState, ConflictsState,
        DiffState, MovingBookmarkState, RebaseState, SquashState,
    };
    use ratatui::crossterm::event::KeyEvent;

    fn ctx<'a>(
        mode: &'a ModeState,
        pending: Option<char>,
        vh: usize,
        focus: bool,
        sel: bool,
    ) -> ControllerContext<'a> {
        ControllerContext {
            mode,
            pending_key: pending,
            viewport_height: vh,
            has_focus: focus,
            has_selection: sel,
        }
    }

    #[test]
    fn test_dispatch_ctrl_u_pages_up() {
        let mode = ModeState::Normal;
        let action = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        );
        assert_eq!(action, Action::PageUp(10));
    }

    #[test]
    fn test_dispatch_d_vs_d() {
        let mode = ModeState::Normal;
        let action_d = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
        );
        assert_eq!(action_d, Action::EnterDiffView);

        let action_big_d = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE),
        );
        assert_eq!(action_big_d, Action::EditDescription);
    }

    #[test]
    fn test_dispatch_normal_esc_conditional() {
        let mode = ModeState::Normal;

        let a = handle_key(
            &ctx(&mode, None, 20, true, false),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert_eq!(a, Action::Unfocus);

        let b = handle_key(
            &ctx(&mode, None, 20, false, true),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert_eq!(b, Action::ClearSelection);

        let c = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert_eq!(c, Action::Noop);
    }

    #[test]
    fn test_dispatch_pending_chord() {
        let mode = ModeState::Normal;

        let action = handle_key(
            &ctx(&mode, Some('g'), 20, false, false),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
        );
        assert_eq!(action, Action::GitImport);

        let unknown = handle_key(
            &ctx(&mode, Some('g'), 20, false, false),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );
        assert_eq!(unknown, Action::ClearPendingKey);
    }

    #[test]
    fn test_dispatch_bookmark_input_any_char() {
        let state = BookmarkInputState {
            name: String::new(),
            cursor: 0,
            target_rev: "aaaa".to_string(),
            deleting: false,
        };
        let mode = ModeState::BookmarkInput(state);
        let action = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );
        assert_eq!(action, Action::BookmarkInputChar('x'));
    }

    #[test]
    fn test_dispatch_bookmark_picker_arrows() {
        let state = BookmarkPickerState {
            all_bookmarks: vec![],
            filter: String::new(),
            filter_cursor: 0,
            selected_index: 0,
            target_rev: "aaaa".to_string(),
            action: BookmarkSelectAction::Move,
        };
        let mode = ModeState::BookmarkPicker(state);
        let down = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert_eq!(down, Action::BookmarkPickerDown);
    }

    #[test]
    fn test_dispatch_conflicts_resolve() {
        let mode = ModeState::Conflicts(ConflictsState::default());
        let action = handle_key(
            &ctx(&mode, None, 20, false, false),
            KeyEvent::new(KeyCode::Char('R'), KeyModifiers::NONE),
        );
        assert_eq!(action, Action::StartResolveFromConflicts);
    }

    #[test]
    fn test_hint_contains_d_desc() {
        let hints = status_bar_hints(&StatusHintContext {
            mode: ModeId::Normal,
            has_selection: false,
            has_focus: false,
            current_has_bookmark: false,
            rebase_allow_branches: None,
        });
        assert!(
            hints.contains("D:desc"),
            "expected `D:desc` in hints; got: {hints}"
        );
    }

    #[test]
    fn test_hint_bookmark_picker_uses_arrows() {
        let hints = status_bar_hints(&StatusHintContext {
            mode: ModeId::BookmarkPicker,
            has_selection: false,
            has_focus: false,
            current_has_bookmark: false,
            rebase_allow_branches: None,
        });
        assert!(
            hints.contains("↑/↓:navigate"),
            "expected arrow navigation; got: {hints}"
        );
    }

    #[test]
    fn test_hint_diff_has_zt_zb() {
        let hints = status_bar_hints(&StatusHintContext {
            mode: ModeId::Diff,
            has_selection: false,
            has_focus: false,
            current_has_bookmark: false,
            rebase_allow_branches: None,
        });
        assert!(
            hints.contains("zt/zb:top/bottom"),
            "expected `zt/zb:top/bottom`; got: {hints}"
        );
    }

    #[test]
    fn test_help_contains_ctrl_d_and_g_f() {
        let view = build_help_view();
        let text: String = view
            .iter()
            .flat_map(|s| {
                s.items
                    .iter()
                    .map(|i| format!("{} {}", i.keys, i.description))
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Ctrl+d"), "expected Ctrl+d; got:\n{text}");
        assert!(text.contains("g f"), "expected g f; got:\n{text}");
    }

    // Silence unused imports if the file layout changes
    #[allow(dead_code)]
    fn _dummy(
        _d: DiffState,
        _c: ConfirmState,
        _r: RebaseState,
        _s: SquashState,
        _m: MovingBookmarkState,
        _a: ConfirmAction,
    ) {
    }
}
