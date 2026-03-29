use super::super::action::Action;
use super::super::controller::ControllerContext;
use super::super::state::ModeState;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyPattern {
    Exact {
        code: KeyCode,
        required_mods: KeyModifiers,
    },
    AnyChar,
}

impl KeyPattern {
    pub(crate) fn matches(&self, event: &KeyEvent) -> Option<MatchCapture> {
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
pub(crate) enum MatchCapture {
    None,
    Char(char),
}

impl MatchCapture {
    pub(crate) fn char(self) -> Option<char> {
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
    pub(crate) fn build(&self, ctx: &ControllerContext<'_>, captured: Option<char>) -> Action {
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

#[derive(Debug)]
pub struct Binding {
    pub mode: ModeId,
    pub pending_prefix: Option<char>,
    pub key: KeyPattern,
    pub action: ActionTemplate,
    pub display: DisplayKind,
    pub label: &'static str,
    pub help: Option<(&'static str, &'static str)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyDef {
    Char(char),
    Ctrl(char),
    Key(KeyCode),
    AnyChar,
}

impl KeyDef {
    pub(crate) const fn to_pattern(self) -> KeyPattern {
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

    pub(crate) const fn plain_char(self) -> Option<char> {
        match self {
            KeyDef::Char(c) => Some(c),
            KeyDef::Ctrl(_) | KeyDef::Key(_) | KeyDef::AnyChar => None,
        }
    }
}
