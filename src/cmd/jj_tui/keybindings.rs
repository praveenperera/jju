//! Centralized keymap and keybinding rendering.
//!
//! This module is the single source of truth for:
//! - Mapping (ModeState, KeyEvent) -> Action
//! - Status bar / context help hints
//! - Prefix (chord) menus
//! - Help popup content

mod dispatch;
mod display;
mod help;
mod hints;

use std::sync::OnceLock;

use super::action::Action;
use super::controller::ControllerContext;
use super::state::{BookmarkSelectAction, ModeState, RebaseType};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) use dispatch::dispatch_key;
pub(crate) use display::{KeyFormat, display_keys_joined, prefix_menu};
pub(crate) use help::build_help_view;
pub(crate) use hints::{StatusHintContext, status_bar_hints};

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
    BookmarkSelect,
    BookmarkPicker,
    PushSelect,
    Conflicts,
}

pub fn mode_id_from_state(mode: &ModeState) -> ModeId {
    match mode {
        ModeState::Normal => ModeId::Normal,
        ModeState::Help(..) => ModeId::Help,
        ModeState::ViewingDiff(_) => ModeId::Diff,
        ModeState::Confirming(_) => ModeId::Confirm,
        ModeState::Selecting => ModeId::Selecting,
        ModeState::Rebasing(_) => ModeId::Rebase,
        ModeState::Squashing(_) => ModeId::Squash,
        ModeState::MovingBookmark(_) => ModeId::MovingBookmark,
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
    BindingDef::new(Normal, Char('R'), act!(RefreshTree), "refresh").help("View", "Refresh tree"),
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
    BindingDef::new(Normal, Char('m'), act!(EnterMoveBookmarkMode), "set")
        .prefix('b')
        .help("Bookmarks & Git", "Set/move bookmark"),
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
    BindingDef::new(Help, Char('j'), act!(ScrollHelpDown(1)), "scroll-down"),
    BindingDef::new(
        Help,
        Key(KeyCode::Down),
        act!(ScrollHelpDown(1)),
        "scroll-down",
    )
    .alias(),
    BindingDef::new(Help, Char('k'), act!(ScrollHelpUp(1)), "scroll-up"),
    BindingDef::new(Help, Key(KeyCode::Up), act!(ScrollHelpUp(1)), "scroll-up").alias(),
    BindingDef::new(Help, Char('d'), act!(ScrollHelpDown(20)), "page-down"),
    BindingDef::new(Help, Char('u'), act!(ScrollHelpUp(20)), "page-up"),
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::controller::{ControllerContext, handle_key};
    use crate::cmd::jj_tui::state::{
        BookmarkPickerState, ConfirmAction, ConfirmState, ConflictsState, DiffState,
        MovingBookmarkState, RebaseState, SquashState,
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
