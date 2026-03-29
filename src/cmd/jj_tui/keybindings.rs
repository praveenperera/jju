//! Centralized keymap and keybinding rendering.
//!
//! This module is the single source of truth for:
//! - Mapping (ModeState, KeyEvent) -> Action
//! - Status bar / context help hints
//! - Prefix (chord) menus
//! - Help popup content

mod bindings;
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

static BINDINGS: OnceLock<Vec<Binding>> = OnceLock::new();

fn bindings() -> &'static [Binding] {
    BINDINGS.get_or_init(|| {
        bindings::binding_defs()
            .iter()
            .map(BindingDef::to_binding)
            .collect()
    })
}

pub fn prefix_title(prefix: char) -> Option<&'static str> {
    bindings::prefix_title(prefix)
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
    use crate::cmd::jj_tui::state::{BookmarkPickerState, ConflictsState};
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
}
