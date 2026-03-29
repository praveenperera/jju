//! Centralized keymap and keybinding rendering.
//!
//! This module is the single source of truth for:
//! - Mapping (ModeState, KeyEvent) -> Action
//! - Status bar / context help hints
//! - Prefix (chord) menus
//! - Help popup content

mod bindings;
mod config;
mod dispatch;
mod display;
mod help;
mod hints;
mod registry;
mod spec;
mod types;

pub(crate) use dispatch::dispatch_key;
pub(crate) use display::{KeyFormat, display_keys_joined, prefix_menu};
pub(crate) use help::build_help_view;
pub(crate) use hints::{StatusHintContext, status_bar_hints};
pub(crate) use registry::{bindings, initialize, warning_duration};
pub use registry::{is_known_prefix, prefix_title};
pub(crate) use spec::{BindingBehavior, BindingSpec, KeySequence};
pub(crate) use types::{ActionTemplate, Binding, DisplayKind, KeyDef, KeyPattern};
pub use types::{ModeId, mode_id_from_state};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::action::Action;
    use crate::cmd::jj_tui::controller::{ControllerContext, handle_key};
    use crate::cmd::jj_tui::state::{
        BookmarkPickerState, BookmarkSelectAction, ConflictsState, ModeState,
    };
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
            .flat_map(|section| {
                section
                    .items
                    .iter()
                    .map(|item| format!("{} {}", item.keys, item.description))
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Ctrl+d"), "expected Ctrl+d; got:\n{text}");
        assert!(text.contains("g f"), "expected g f; got:\n{text}");
    }
}
