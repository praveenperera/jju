//! Centralized keymap and keybinding rendering.
//!
//! This module is the single source of truth for:
//! - Mapping (ModeState, KeyEvent) -> Action
//! - Status bar / context help hints
//! - Prefix (chord) menus
//! - Help popup content

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
        ModeState::Conflicts(_) => ModeId::Conflicts,
    }
}

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

#[allow(dead_code)]
pub struct KeySeq(pub &'static [KeyPattern]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTemplate {
    Fixed(Action),
    PageUpHalfViewport,
    PageDownHalfViewport,
    CenterCursorViewport,
    BookmarkInputChar,
    BookmarkFilterChar,
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
            ActionTemplate::BookmarkFilterChar => Action::BookmarkFilterChar(captured.unwrap_or(' ')),
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
}

pub const PREFIX_TITLES: &[(char, &'static str)] = &[('g', "git"), ('z', "nav"), ('b', "bookmark")];

pub fn prefix_title(prefix: char) -> Option<&'static str> {
    PREFIX_TITLES.iter().find(|(p, _)| *p == prefix).map(|(_, t)| *t)
}

pub fn is_known_prefix(prefix: char) -> bool {
    prefix_title(prefix).is_some()
}

// The single source of truth for behavior and displayed keys.
pub static DEFAULT_BINDINGS: &[Binding] = &[
    // Normal
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('q'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::Quit),
        display: DisplayKind::Primary,
        label: "quit",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('c'),
            required_mods: KeyModifiers::CONTROL,
        },
        action: ActionTemplate::Fixed(Action::Quit),
        display: DisplayKind::Alias,
        label: "quit",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('Q'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterSquashMode),
        display: DisplayKind::Primary,
        label: "squash",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::NormalEscConditional,
        display: DisplayKind::Primary,
        label: "esc",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('?'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterHelp),
        display: DisplayKind::Primary,
        label: "help",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorDown),
        display: DisplayKind::Primary,
        label: "down",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorDown),
        display: DisplayKind::Alias,
        label: "down",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorUp),
        display: DisplayKind::Primary,
        label: "up",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorUp),
        display: DisplayKind::Alias,
        label: "up",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('@'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::JumpToWorkingCopy),
        display: DisplayKind::Primary,
        label: "working-copy",
    },
    // Prefix keys
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('g'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SetPendingKey('g')),
        display: DisplayKind::Primary,
        label: "git",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('z'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SetPendingKey('z')),
        display: DisplayKind::Primary,
        label: "nav",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('b'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SetPendingKey('b')),
        display: DisplayKind::Primary,
        label: "bookmark",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('f'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleFullMode),
        display: DisplayKind::Primary,
        label: "full",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleFocus),
        display: DisplayKind::Primary,
        label: "zoom",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Tab,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleExpanded),
        display: DisplayKind::Primary,
        label: "details",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char(' '),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleExpanded),
        display: DisplayKind::Alias,
        label: "details",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('u'),
            required_mods: KeyModifiers::CONTROL,
        },
        action: ActionTemplate::PageUpHalfViewport,
        display: DisplayKind::Primary,
        label: "page-up",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('d'),
            required_mods: KeyModifiers::CONTROL,
        },
        action: ActionTemplate::PageDownHalfViewport,
        display: DisplayKind::Primary,
        label: "page-down",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('\\'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleSplitView),
        display: DisplayKind::Primary,
        label: "split",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('d'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterDiffView),
        display: DisplayKind::Primary,
        label: "diff",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('D'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EditDescription),
        display: DisplayKind::Primary,
        label: "desc",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('e'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EditWorkingCopy),
        display: DisplayKind::Primary,
        label: "edit",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('n'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::CreateNewCommit),
        display: DisplayKind::Primary,
        label: "new",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('c'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::CommitWorkingCopy),
        display: DisplayKind::Primary,
        label: "commit",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('x'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleSelection),
        display: DisplayKind::Primary,
        label: "toggle",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('v'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterSelecting),
        display: DisplayKind::Primary,
        label: "select",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('a'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterConfirmAbandon),
        display: DisplayKind::Primary,
        label: "abandon",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('r'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterRebaseMode(RebaseType::Single)),
        display: DisplayKind::Primary,
        label: "rebase-single",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('s'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterRebaseMode(RebaseType::WithDescendants)),
        display: DisplayKind::Primary,
        label: "rebase-desc",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('t'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterConfirmRebaseOntoTrunk(RebaseType::Single)),
        display: DisplayKind::Primary,
        label: "trunk-single",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('T'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterConfirmRebaseOntoTrunk(RebaseType::WithDescendants)),
        display: DisplayKind::Primary,
        label: "trunk-desc",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('u'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::Undo),
        display: DisplayKind::Primary,
        label: "undo",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('p'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::GitPush),
        display: DisplayKind::Primary,
        label: "push",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('P'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::GitPushAll),
        display: DisplayKind::Primary,
        label: "push-all",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('C'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterConflicts),
        display: DisplayKind::Primary,
        label: "conflicts",
    },

    // Normal chords: g
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('g'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('f'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::GitFetch),
        display: DisplayKind::Primary,
        label: "fetch",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('g'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('i'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::GitImport),
        display: DisplayKind::Primary,
        label: "import",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('g'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('e'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::GitExport),
        display: DisplayKind::Primary,
        label: "export",
    },
    // Normal chords: z
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('z'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('t'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorTop),
        display: DisplayKind::Primary,
        label: "top",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('z'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('b'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorBottom),
        display: DisplayKind::Primary,
        label: "bottom",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('z'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('z'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::CenterCursorViewport,
        display: DisplayKind::Primary,
        label: "center",
    },
    // Normal chords: b (bookmarks)
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('b'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('m'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterMoveBookmarkMode),
        display: DisplayKind::Primary,
        label: "move",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('b'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('s'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterCreateBookmark),
        display: DisplayKind::Primary,
        label: "set/new",
    },
    Binding {
        mode: ModeId::Normal,
        pending_prefix: Some('b'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('d'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterBookmarkPicker(BookmarkSelectAction::Delete)),
        display: DisplayKind::Primary,
        label: "delete",
    },

    // Help mode
    Binding {
        mode: ModeId::Help,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('q'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitHelp),
        display: DisplayKind::Primary,
        label: "close",
    },
    Binding {
        mode: ModeId::Help,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('?'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitHelp),
        display: DisplayKind::Alias,
        label: "close",
    },
    Binding {
        mode: ModeId::Help,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitHelp),
        display: DisplayKind::Alias,
        label: "close",
    },

    // Diff
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffDown(1)),
        display: DisplayKind::Primary,
        label: "scroll-down",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffDown(1)),
        display: DisplayKind::Alias,
        label: "scroll-down",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffUp(1)),
        display: DisplayKind::Primary,
        label: "scroll-up",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffUp(1)),
        display: DisplayKind::Alias,
        label: "scroll-up",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('d'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffDown(20)),
        display: DisplayKind::Primary,
        label: "page-down",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('u'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffUp(20)),
        display: DisplayKind::Primary,
        label: "page-up",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('z'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SetPendingKey('z')),
        display: DisplayKind::Primary,
        label: "nav",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('q'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitDiffView),
        display: DisplayKind::Primary,
        label: "close",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitDiffView),
        display: DisplayKind::Alias,
        label: "close",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: Some('z'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('t'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffTop),
        display: DisplayKind::Primary,
        label: "top",
    },
    Binding {
        mode: ModeId::Diff,
        pending_prefix: Some('z'),
        key: KeyPattern::Exact {
            code: KeyCode::Char('b'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ScrollDiffBottom),
        display: DisplayKind::Primary,
        label: "bottom",
    },

    // Confirm
    Binding {
        mode: ModeId::Confirm,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('y'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmYes),
        display: DisplayKind::Primary,
        label: "yes",
    },
    Binding {
        mode: ModeId::Confirm,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmYes),
        display: DisplayKind::Alias,
        label: "yes",
    },
    Binding {
        mode: ModeId::Confirm,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('n'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmNo),
        display: DisplayKind::Primary,
        label: "no",
    },
    Binding {
        mode: ModeId::Confirm,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmNo),
        display: DisplayKind::Alias,
        label: "no",
    },

    // Selecting
    Binding {
        mode: ModeId::Selecting,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorDown),
        display: DisplayKind::Primary,
        label: "down",
    },
    Binding {
        mode: ModeId::Selecting,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorDown),
        display: DisplayKind::Alias,
        label: "down",
    },
    Binding {
        mode: ModeId::Selecting,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorUp),
        display: DisplayKind::Primary,
        label: "up",
    },
    Binding {
        mode: ModeId::Selecting,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveCursorUp),
        display: DisplayKind::Alias,
        label: "up",
    },
    Binding {
        mode: ModeId::Selecting,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitSelecting),
        display: DisplayKind::Primary,
        label: "exit",
    },
    Binding {
        mode: ModeId::Selecting,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('a'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::EnterConfirmAbandon),
        display: DisplayKind::Primary,
        label: "abandon",
    },

    // Rebase
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveRebaseDestDown),
        display: DisplayKind::Primary,
        label: "dest-down",
    },
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveRebaseDestDown),
        display: DisplayKind::Alias,
        label: "dest-down",
    },
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveRebaseDestUp),
        display: DisplayKind::Primary,
        label: "dest-up",
    },
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveRebaseDestUp),
        display: DisplayKind::Alias,
        label: "dest-up",
    },
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('b'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ToggleRebaseBranches),
        display: DisplayKind::Primary,
        label: "branches",
    },
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExecuteRebase),
        display: DisplayKind::Primary,
        label: "run",
    },
    Binding {
        mode: ModeId::Rebase,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitRebaseMode),
        display: DisplayKind::Primary,
        label: "cancel",
    },

    // Squash
    Binding {
        mode: ModeId::Squash,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveSquashDestDown),
        display: DisplayKind::Primary,
        label: "dest-down",
    },
    Binding {
        mode: ModeId::Squash,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveSquashDestDown),
        display: DisplayKind::Alias,
        label: "dest-down",
    },
    Binding {
        mode: ModeId::Squash,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveSquashDestUp),
        display: DisplayKind::Primary,
        label: "dest-up",
    },
    Binding {
        mode: ModeId::Squash,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveSquashDestUp),
        display: DisplayKind::Alias,
        label: "dest-up",
    },
    Binding {
        mode: ModeId::Squash,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExecuteSquash),
        display: DisplayKind::Primary,
        label: "run",
    },
    Binding {
        mode: ModeId::Squash,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitSquashMode),
        display: DisplayKind::Primary,
        label: "cancel",
    },

    // MovingBookmark
    Binding {
        mode: ModeId::MovingBookmark,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveBookmarkDestDown),
        display: DisplayKind::Primary,
        label: "dest-down",
    },
    Binding {
        mode: ModeId::MovingBookmark,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveBookmarkDestDown),
        display: DisplayKind::Alias,
        label: "dest-down",
    },
    Binding {
        mode: ModeId::MovingBookmark,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveBookmarkDestUp),
        display: DisplayKind::Primary,
        label: "dest-up",
    },
    Binding {
        mode: ModeId::MovingBookmark,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::MoveBookmarkDestUp),
        display: DisplayKind::Alias,
        label: "dest-up",
    },
    Binding {
        mode: ModeId::MovingBookmark,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExecuteBookmarkMove),
        display: DisplayKind::Primary,
        label: "run",
    },
    Binding {
        mode: ModeId::MovingBookmark,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitBookmarkMode),
        display: DisplayKind::Primary,
        label: "cancel",
    },

    // BookmarkInput
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmBookmarkInput),
        display: DisplayKind::Primary,
        label: "confirm",
    },
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitBookmarkMode),
        display: DisplayKind::Primary,
        label: "cancel",
    },
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Backspace,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkInputBackspace),
        display: DisplayKind::Alias,
        label: "backspace",
    },
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Delete,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkInputDelete),
        display: DisplayKind::Alias,
        label: "delete",
    },
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Left,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkInputCursorLeft),
        display: DisplayKind::Alias,
        label: "left",
    },
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Right,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkInputCursorRight),
        display: DisplayKind::Alias,
        label: "right",
    },
    Binding {
        mode: ModeId::BookmarkInput,
        pending_prefix: None,
        key: KeyPattern::AnyChar,
        action: ActionTemplate::BookmarkInputChar,
        display: DisplayKind::Primary,
        label: "type",
    },

    // BookmarkSelect
    Binding {
        mode: ModeId::BookmarkSelect,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SelectBookmarkDown),
        display: DisplayKind::Primary,
        label: "down",
    },
    Binding {
        mode: ModeId::BookmarkSelect,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SelectBookmarkDown),
        display: DisplayKind::Alias,
        label: "down",
    },
    Binding {
        mode: ModeId::BookmarkSelect,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SelectBookmarkUp),
        display: DisplayKind::Primary,
        label: "up",
    },
    Binding {
        mode: ModeId::BookmarkSelect,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::SelectBookmarkUp),
        display: DisplayKind::Alias,
        label: "up",
    },
    Binding {
        mode: ModeId::BookmarkSelect,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmBookmarkSelect),
        display: DisplayKind::Primary,
        label: "select",
    },
    Binding {
        mode: ModeId::BookmarkSelect,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitBookmarkMode),
        display: DisplayKind::Primary,
        label: "cancel",
    },

    // BookmarkPicker
    Binding {
        mode: ModeId::BookmarkPicker,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitBookmarkMode),
        display: DisplayKind::Primary,
        label: "cancel",
    },
    Binding {
        mode: ModeId::BookmarkPicker,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConfirmBookmarkPicker),
        display: DisplayKind::Primary,
        label: "confirm",
    },
    Binding {
        mode: ModeId::BookmarkPicker,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkPickerDown),
        display: DisplayKind::Primary,
        label: "down",
    },
    Binding {
        mode: ModeId::BookmarkPicker,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkPickerUp),
        display: DisplayKind::Primary,
        label: "up",
    },
    Binding {
        mode: ModeId::BookmarkPicker,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Backspace,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::BookmarkFilterBackspace),
        display: DisplayKind::Alias,
        label: "backspace",
    },
    Binding {
        mode: ModeId::BookmarkPicker,
        pending_prefix: None,
        key: KeyPattern::AnyChar,
        action: ActionTemplate::BookmarkFilterChar,
        display: DisplayKind::Primary,
        label: "type",
    },

    // Conflicts
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('j'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConflictsDown),
        display: DisplayKind::Primary,
        label: "down",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Down,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConflictsDown),
        display: DisplayKind::Alias,
        label: "down",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('k'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConflictsUp),
        display: DisplayKind::Primary,
        label: "up",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Up,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConflictsUp),
        display: DisplayKind::Alias,
        label: "up",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Enter,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ConflictsJump),
        display: DisplayKind::Primary,
        label: "jump",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('R'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::StartResolveFromConflicts),
        display: DisplayKind::Primary,
        label: "resolve",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Char('q'),
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitConflicts),
        display: DisplayKind::Primary,
        label: "exit",
    },
    Binding {
        mode: ModeId::Conflicts,
        pending_prefix: None,
        key: KeyPattern::Exact {
            code: KeyCode::Esc,
            required_mods: KeyModifiers::NONE,
        },
        action: ActionTemplate::Fixed(Action::ExitConflicts),
        display: DisplayKind::Alias,
        label: "exit",
    },
];

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
    for binding in DEFAULT_BINDINGS {
        if binding.mode != mode || binding.pending_prefix != pending_prefix {
            continue;
        }
        if let Some(captured) = binding.key.matches(key) {
            return Some(binding.action.build(ctx, captured.char()));
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct PrefixMenuView {
    pub title: &'static str,
    pub items: Vec<(String, &'static str)>,
}

pub fn prefix_menu(mode: ModeId, pending: char) -> Option<PrefixMenuView> {
    let title = prefix_title(pending)?;
    let mut items = Vec::new();
    for binding in DEFAULT_BINDINGS {
        if binding.mode != mode || binding.pending_prefix != Some(pending) {
            continue;
        }
        if binding.display != DisplayKind::Primary {
            continue;
        }
        items.push((format_binding_key(binding, KeyFormat::SecondKeyOnly), binding.label));
    }
    Some(PrefixMenuView { title, items })
}

#[derive(Debug, Clone, Copy)]
pub enum KeyFormat {
    Space,
    Concat,
    SecondKeyOnly,
}

pub fn format_binding_key(binding: &Binding, fmt: KeyFormat) -> String {
    let key = display_key_pattern(&binding.key, binding.display);
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

fn display_key_pattern(key: &KeyPattern, _display: DisplayKind) -> String {
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

fn find_bindings(mode: ModeId, pending: Option<char>, label: &str) -> impl Iterator<Item = &'static Binding> {
    DEFAULT_BINDINGS
        .iter()
        .filter(move |b| b.mode == mode && b.pending_prefix == pending && b.label == label)
}

fn first_key(mode: ModeId, pending: Option<char>, label: &str, kind: DisplayKind) -> Option<String> {
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
            (Some(_), KeyFormat::SecondKeyOnly) => format_binding_key(binding, KeyFormat::SecondKeyOnly),
            (Some(_), KeyFormat::Concat) => format_binding_key(binding, KeyFormat::Concat),
            (Some(_), KeyFormat::Space) => format_binding_key(binding, KeyFormat::Space),
            (None, _) => display_key_pattern(&binding.key, binding.display),
        };
        keys.push(k);
    }
    keys
}

fn join_keys(keys: &[String], sep: &str) -> String {
    keys.join(sep)
}

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

#[derive(Debug, Clone, Copy)]
pub struct HelpItemSpec {
    pub mode: ModeId,
    pub pending_prefix: Option<char>,
    pub label: &'static str,
    pub include_aliases: bool,
    pub chord_format: KeyFormat,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct HelpSectionSpec {
    pub title: &'static str,
    pub items: &'static [HelpItemSpec],
}

pub static HELP_SECTIONS: &[HelpSectionSpec] = &[
    HelpSectionSpec {
        title: "Navigation",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "down",
                include_aliases: true,
                chord_format: KeyFormat::Space,
                description: "Move cursor down",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "up",
                include_aliases: true,
                chord_format: KeyFormat::Space,
                description: "Move cursor up",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "page-down",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Page down",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "page-up",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Page up",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('z'),
                label: "top",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Jump to top",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('z'),
                label: "bottom",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Jump to bottom",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('z'),
                label: "center",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Center current line",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "working-copy",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Jump to working copy",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "zoom",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Zoom in/out on node",
            },
        ],
    },
    HelpSectionSpec {
        title: "View",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "desc",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Edit description",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "diff",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "View diff",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "details",
                include_aliases: true,
                chord_format: KeyFormat::Space,
                description: "Toggle commit details",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "split",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Toggle split view",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "full",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Toggle full mode",
            },
        ],
    },
    HelpSectionSpec {
        title: "Edit Operations",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "edit",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Edit working copy (jj edit)",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "new",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "New commit (jj new)",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "commit",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Commit changes (jj commit)",
            },
        ],
    },
    HelpSectionSpec {
        title: "Selection",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "toggle",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Toggle selection",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "select",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Visual select mode",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "abandon",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Abandon selected",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "esc",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Clear selection (when selected)",
            },
        ],
    },
    HelpSectionSpec {
        title: "Rebase",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "rebase-single",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Rebase single (-r)",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "rebase-desc",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Rebase + descendants (-s)",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "trunk-single",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Quick rebase onto trunk",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "trunk-desc",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Quick rebase tree onto trunk",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "squash",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Squash into target",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "undo",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Undo last operation",
            },
        ],
    },
    HelpSectionSpec {
        title: "Bookmarks & Git",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "push",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Push current bookmark",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('b'),
                label: "move",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Move bookmark",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('b'),
                label: "set/new",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Set/create bookmark",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('b'),
                label: "delete",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Delete bookmark",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('g'),
                label: "fetch",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Git fetch",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('g'),
                label: "import",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Git import",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: Some('g'),
                label: "export",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Git export",
            },
        ],
    },
    HelpSectionSpec {
        title: "Conflicts",
        items: &[HelpItemSpec {
            mode: ModeId::Normal,
            pending_prefix: None,
            label: "conflicts",
            include_aliases: false,
            chord_format: KeyFormat::Space,
            description: "View conflicts panel",
        }],
    },
    HelpSectionSpec {
        title: "General",
        items: &[
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "help",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Toggle help",
            },
            HelpItemSpec {
                mode: ModeId::Normal,
                pending_prefix: None,
                label: "quit",
                include_aliases: false,
                chord_format: KeyFormat::Space,
                description: "Quit",
            },
        ],
    },
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
    let mut out = Vec::new();
    for section in HELP_SECTIONS {
        let mut items = Vec::new();
        for item in section.items {
            let keys = if item.pending_prefix.is_some() {
                keys_for_label(item.mode, item.pending_prefix, item.label, item.include_aliases, item.chord_format)
            } else {
                keys_for_label(item.mode, None, item.label, item.include_aliases, item.chord_format)
            };
            let keys = if item.include_aliases {
                join_keys(&keys, "/")
            } else {
                keys.into_iter().next().unwrap_or_else(|| "?".to_string())
            };
            items.push(HelpItemView {
                keys,
                description: item.description,
            });
        }
        out.push(HelpSectionView {
            title: section.title,
            items,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::state::{
        BookmarkInputState, BookmarkPickerState, ConfirmAction, ConfirmState, ConflictsState,
        DiffState, MovingBookmarkState, RebaseState, SquashState,
    };
    use ratatui::crossterm::event::KeyEvent;

    fn ctx<'a>(mode: &'a ModeState, pending: Option<char>, vh: usize, focus: bool, sel: bool) -> ControllerContext<'a> {
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
            .flat_map(|s| s.items.iter().map(|i| format!("{} {}", i.keys, i.description)))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Ctrl+d"), "expected Ctrl+d; got:\n{text}");
        assert!(text.contains("g f"), "expected g f; got:\n{text}");
    }

    // Silence unused imports if the file layout changes.
    #[allow(dead_code)]
    fn _dummy(_d: DiffState, _c: ConfirmState, _r: RebaseState, _s: SquashState, _m: MovingBookmarkState, _a: ConfirmAction) {}
}
