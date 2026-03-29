//! Engine for jj_tui
//!
//! The engine is a pure function that processes actions and produces effects.
//! It mutates state but performs no IO.

mod bookmarks;
mod commands;
mod modes;
mod navigation;
mod rebase;
mod selection;

use super::action::Action;
use super::effect::Effect;
use super::state::{MessageKind, ModeState, PendingOperation};
use super::tree::TreeState;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct ReduceCtx<'a> {
    pub tree: &'a mut TreeState,
    pub mode: &'a mut ModeState,
    pub should_quit: &'a mut bool,
    pub split_view: &'a mut bool,
    pub pending_key: &'a mut Option<char>,
    pub pending_operation: &'a mut Option<PendingOperation>,
    pub syntax_set: &'a SyntaxSet,
    pub theme_set: &'a ThemeSet,
    pub effects: Vec<Effect>,
}

pub struct ReduceResources<'a> {
    pub syntax_set: &'a SyntaxSet,
    pub theme_set: &'a ThemeSet,
}

impl<'a> ReduceCtx<'a> {
    pub fn new(
        tree: &'a mut TreeState,
        mode: &'a mut ModeState,
        should_quit: &'a mut bool,
        split_view: &'a mut bool,
        pending_key: &'a mut Option<char>,
        pending_operation: &'a mut Option<PendingOperation>,
        resources: ReduceResources<'a>,
    ) -> Self {
        Self {
            tree,
            mode,
            should_quit,
            split_view,
            pending_key,
            pending_operation,
            syntax_set: resources.syntax_set,
            theme_set: resources.theme_set,
            effects: Vec::new(),
        }
    }

    fn set_status(&mut self, text: impl Into<String>, kind: MessageKind) {
        self.effects.push(Effect::SetStatus {
            text: text.into(),
            kind,
        });
    }
}

/// Process an action and produce effects
/// Returns effects to be executed by the runner
pub fn reduce(mut ctx: ReduceCtx<'_>, action: Action) -> Vec<Effect> {
    let clear_pending_key = !matches!(&action, Action::SetPendingKey(_));

    match action {
        Action::Quit => *ctx.should_quit = true,
        Action::Noop => {}
        Action::SetPendingKey(prefix) => *ctx.pending_key = Some(prefix),
        Action::ClearPendingKey => *ctx.pending_key = None,
        Action::MoveCursorUp
        | Action::MoveCursorDown
        | Action::MoveCursorTop
        | Action::MoveCursorBottom
        | Action::JumpToWorkingCopy
        | Action::PageUp(_)
        | Action::PageDown(_)
        | Action::CenterCursor(_)
        | Action::ToggleFocus
        | Action::Unfocus
        | Action::ToggleExpanded
        | Action::ToggleFullMode
        | Action::ToggleSplitView
        | Action::EnterHelp
        | Action::ExitHelp
        | Action::ScrollHelpUp(_)
        | Action::ScrollHelpDown(_)
        | Action::EnterSelecting
        | Action::ExitSelecting
        | Action::ToggleSelection
        | Action::ClearSelection
        | Action::RefreshTree => navigation::handle(&mut ctx, action),
        Action::EnterDiffView
        | Action::ExitDiffView
        | Action::EnterConfirmAbandon
        | Action::EnterConfirmStackSync
        | Action::EnterConfirmRebaseOntoTrunk(_)
        | Action::ConfirmYes
        | Action::ConfirmNo
        | Action::EnterRebaseMode(_)
        | Action::ExitRebaseMode
        | Action::MoveRebaseDestUp
        | Action::MoveRebaseDestDown
        | Action::ToggleRebaseBranches
        | Action::ExecuteRebase
        | Action::EnterSquashMode
        | Action::ExitSquashMode
        | Action::MoveSquashDestUp
        | Action::MoveSquashDestDown
        | Action::ExecuteSquash
        | Action::ScrollDiffUp(_)
        | Action::ScrollDiffDown(_)
        | Action::ScrollDiffTop
        | Action::ScrollDiffBottom
        | Action::EnterConflicts
        | Action::ExitConflicts
        | Action::ConflictsUp
        | Action::ConflictsDown
        | Action::ConflictsJump
        | Action::StartResolveFromConflicts => modes::handle(&mut ctx, action),
        Action::EnterMoveBookmarkMode
        | Action::EnterBookmarkPicker(_)
        | Action::ExitBookmarkMode
        | Action::MoveBookmarkDestUp
        | Action::MoveBookmarkDestDown
        | Action::ExecuteBookmarkMove
        | Action::SelectBookmarkUp
        | Action::SelectBookmarkDown
        | Action::ConfirmBookmarkSelect
        | Action::BookmarkPickerUp
        | Action::BookmarkPickerDown
        | Action::BookmarkFilterChar(_)
        | Action::BookmarkFilterBackspace
        | Action::ConfirmBookmarkPicker
        | Action::GitPush
        | Action::GitPushAll
        | Action::PushSelectUp
        | Action::PushSelectDown
        | Action::PushSelectToggle
        | Action::PushSelectAll
        | Action::PushSelectNone
        | Action::PushSelectFilterChar(_)
        | Action::PushSelectFilterBackspace
        | Action::PushSelectConfirm
        | Action::ExitPushSelect => bookmarks::handle(&mut ctx, action),
        Action::EditWorkingCopy
        | Action::CreateNewCommit
        | Action::CommitWorkingCopy
        | Action::EditDescription
        | Action::Undo
        | Action::GitFetch
        | Action::GitImport
        | Action::GitExport
        | Action::ResolveDivergence
        | Action::CreatePR => commands::handle(&mut ctx, action),
    }

    if clear_pending_key {
        *ctx.pending_key = None;
    }

    ctx.effects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::state::{BookmarkPickerState, BookmarkSelectAction};
    use crate::cmd::jj_tui::test_support::{make_node, make_node_with_bookmarks, make_tree};

    struct TestState {
        tree: TreeState,
        mode: ModeState,
        should_quit: bool,
        split_view: bool,
        pending_key: Option<char>,
        pending_operation: Option<PendingOperation>,
        syntax_set: SyntaxSet,
        theme_set: ThemeSet,
    }

    impl TestState {
        fn new(tree: TreeState) -> Self {
            Self {
                tree,
                mode: ModeState::Normal,
                should_quit: false,
                split_view: false,
                pending_key: None,
                pending_operation: None,
                syntax_set: SyntaxSet::load_defaults_newlines(),
                theme_set: ThemeSet::load_defaults(),
            }
        }

        fn reduce(&mut self, action: Action) -> Vec<Effect> {
            reduce(
                ReduceCtx::new(
                    &mut self.tree,
                    &mut self.mode,
                    &mut self.should_quit,
                    &mut self.split_view,
                    &mut self.pending_key,
                    &mut self.pending_operation,
                    ReduceResources {
                        syntax_set: &self.syntax_set,
                        theme_set: &self.theme_set,
                    },
                ),
                action,
            )
        }
    }

    #[test]
    fn test_cursor_movement() {
        let tree = make_tree(vec![
            make_node("aaaa", 0),
            make_node("bbbb", 1),
            make_node("cccc", 2),
        ]);
        let mut state = TestState::new(tree);

        assert_eq!(state.tree.cursor, 0);

        state.reduce(Action::MoveCursorDown);
        assert_eq!(state.tree.cursor, 1);

        state.reduce(Action::MoveCursorDown);
        assert_eq!(state.tree.cursor, 2);

        // should not go past end
        state.reduce(Action::MoveCursorDown);
        assert_eq!(state.tree.cursor, 2);

        state.reduce(Action::MoveCursorUp);
        assert_eq!(state.tree.cursor, 1);

        state.reduce(Action::MoveCursorTop);
        assert_eq!(state.tree.cursor, 0);

        state.reduce(Action::MoveCursorBottom);
        assert_eq!(state.tree.cursor, 2);
    }

    #[test]
    fn test_mode_transitions() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        assert!(matches!(state.mode, ModeState::Normal));

        state.reduce(Action::EnterHelp);
        assert!(matches!(state.mode, ModeState::Help(..)));

        state.reduce(Action::ExitHelp);
        assert!(matches!(state.mode, ModeState::Normal));

        state.reduce(Action::EnterSelecting);
        assert!(matches!(state.mode, ModeState::Selecting));

        state.reduce(Action::ExitSelecting);
        assert!(matches!(state.mode, ModeState::Normal));
    }

    #[test]
    fn test_quit_action() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        assert!(!state.should_quit);
        state.reduce(Action::Quit);
        assert!(state.should_quit);
    }

    #[test]
    fn test_pending_key() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        assert!(state.pending_key.is_none());

        state.reduce(Action::SetPendingKey('g'));
        assert_eq!(state.pending_key, Some('g'));

        state.reduce(Action::ClearPendingKey);
        assert!(state.pending_key.is_none());
    }

    #[test]
    fn test_move_bookmark_picker_pins_current_bookmarks_and_sorts_rest_by_proximity() {
        let mut tree = make_tree(vec![
            make_node_with_bookmarks("rev0", 0, &["pin2", "pin1"]),
            make_node_with_bookmarks("rev1", 1, &["near1"]),
            make_node("rev2", 1),
            make_node_with_bookmarks("rev3", 1, &["near2"]),
            make_node_with_bookmarks("rev4", 1, &["far"]),
        ]);
        tree.cursor = 0;

        let all_bookmarks = vec![
            "near2".to_string(),
            "pin1".to_string(),
            "far".to_string(),
            "near1".to_string(),
            "pin2".to_string(),
        ];
        let pinned = tree.nodes[0].bookmark_names();

        let ordered = bookmarks::build_move_bookmark_picker_list(all_bookmarks, pinned, &tree);
        assert_eq!(
            ordered,
            vec![
                "pin2".to_string(),
                "pin1".to_string(),
                "near1".to_string(),
                "near2".to_string(),
                "far".to_string(),
            ]
        );
    }

    #[test]
    fn test_confirm_bookmark_picker_move_already_here_enters_move_away_flow() {
        let mut tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["a"])]);
        tree.cursor = 0;

        let mut state = TestState::new(tree);
        state.mode = ModeState::BookmarkPicker(BookmarkPickerState {
            all_bookmarks: vec!["a".to_string()],
            filter: String::new(),
            filter_cursor: 0,
            selected_index: 0,
            target_rev: "rev0".to_string(),
            action: BookmarkSelectAction::Move,
        });

        let effects = state.reduce(Action::ConfirmBookmarkPicker);

        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::SaveOperationForUndo))
        );
        assert!(!effects.iter().any(|e| matches!(
            e,
            Effect::RunBookmarkSet { .. } | Effect::RunBookmarkSetBackwards { .. }
        )));

        assert!(matches!(
            state.mode,
            ModeState::MovingBookmark(ref s)
                if s.bookmark_name == "a" && s.dest_cursor == 0
        ));
    }

    #[test]
    fn test_selection_toggle() {
        let tree = make_tree(vec![make_node("aaaa", 0), make_node("bbbb", 1)]);
        let mut state = TestState::new(tree);

        assert!(state.tree.selected.is_empty());

        state.reduce(Action::ToggleSelection);
        assert!(state.tree.selected.contains(&0));

        state.reduce(Action::ToggleSelection);
        assert!(!state.tree.selected.contains(&0));

        state.tree.cursor = 1;
        state.reduce(Action::ToggleSelection);
        assert!(state.tree.selected.contains(&1));

        state.reduce(Action::ClearSelection);
        assert!(state.tree.selected.is_empty());
    }

    #[test]
    fn test_split_view_toggle() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        assert!(!state.split_view);
        state.reduce(Action::ToggleSplitView);
        assert!(state.split_view);
        state.reduce(Action::ToggleSplitView);
        assert!(!state.split_view);
    }

    #[test]
    fn test_selection_mode_extends_on_move() {
        let tree = make_tree(vec![
            make_node("aaaa", 0),
            make_node("bbbb", 1),
            make_node("cccc", 2),
        ]);
        let mut state = TestState::new(tree);

        // enter selection mode
        state.reduce(Action::EnterSelecting);
        assert!(matches!(state.mode, ModeState::Selecting));
        assert!(state.tree.selected.contains(&0));
        assert_eq!(state.tree.selection_anchor, Some(0));

        // move down should extend selection
        state.reduce(Action::MoveCursorDown);
        assert!(state.tree.selected.contains(&0));
        assert!(state.tree.selected.contains(&1));

        state.reduce(Action::MoveCursorDown);
        assert!(state.tree.selected.contains(&0));
        assert!(state.tree.selected.contains(&1));
        assert!(state.tree.selected.contains(&2));
    }

    #[test]
    fn test_page_navigation() {
        let tree = make_tree(vec![
            make_node("a", 0),
            make_node("b", 0),
            make_node("c", 0),
            make_node("d", 0),
            make_node("e", 0),
            make_node("f", 0),
            make_node("g", 0),
            make_node("h", 0),
            make_node("i", 0),
            make_node("j", 0),
        ]);
        let mut state = TestState::new(tree);

        state.reduce(Action::PageDown(5));
        assert_eq!(state.tree.cursor, 5);

        state.reduce(Action::PageUp(3));
        assert_eq!(state.tree.cursor, 2);

        state.reduce(Action::PageUp(10)); // should clamp to 0
        assert_eq!(state.tree.cursor, 0);

        state.reduce(Action::PageDown(100)); // should clamp to max
        assert_eq!(state.tree.cursor, 9);
    }

    #[test]
    fn test_noop_produces_no_effects() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::Noop);
        assert!(effects.is_empty());
    }

    #[test]
    fn test_refresh_tree_produces_effect() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::RefreshTree);
        assert_eq!(effects.len(), 2);
        assert!(matches!(effects[0], Effect::RefreshTree));
        assert!(matches!(effects[1], Effect::SetStatus { .. }));
    }

    #[test]
    fn test_git_push_single_bookmark_pushes_immediately() {
        let tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["feature"])]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::GitPush);

        assert!(matches!(state.mode, ModeState::Normal));
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::RunGitPush { bookmark } if bookmark == "feature"))
        );
    }

    #[test]
    fn test_git_push_multiple_bookmarks_enters_push_select_mode() {
        let tree = make_tree(vec![make_node_with_bookmarks(
            "rev0",
            0,
            &["feature-a", "feature-b"],
        )]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::GitPush);

        // should enter push select mode, not emit push effect
        assert!(effects.is_empty());
        let ModeState::PushSelect(push_state) = &state.mode else {
            panic!("expected PushSelect mode");
        };
        assert_eq!(push_state.all_bookmarks.len(), 2);
        assert!(push_state.selected.contains(&0));
        assert!(push_state.selected.contains(&1));
    }

    #[test]
    fn test_push_select_toggle_selection() {
        let tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["a", "b", "c"])]);
        let mut state = TestState::new(tree);
        state.reduce(Action::GitPush);

        // all should be selected initially
        let ModeState::PushSelect(push_state) = &state.mode else {
            panic!("expected PushSelect mode");
        };
        assert_eq!(push_state.selected.len(), 3);

        // toggle first one off
        state.reduce(Action::PushSelectToggle);
        let ModeState::PushSelect(push_state) = &state.mode else {
            panic!("expected PushSelect mode");
        };
        assert!(!push_state.selected.contains(&0));
        assert!(push_state.selected.contains(&1));
        assert!(push_state.selected.contains(&2));
    }

    #[test]
    fn test_push_select_confirm_pushes_selected() {
        let tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["a", "b", "c"])]);
        let mut state = TestState::new(tree);
        state.reduce(Action::GitPush);

        // deselect "a" (index 0)
        state.reduce(Action::PushSelectToggle);

        let effects = state.reduce(Action::PushSelectConfirm);

        assert!(matches!(state.mode, ModeState::Normal));
        let push_effect = effects
            .iter()
            .find(|e| matches!(e, Effect::RunGitPushMultiple { .. }));
        assert!(push_effect.is_some());
        if let Some(Effect::RunGitPushMultiple { bookmarks }) = push_effect {
            assert_eq!(bookmarks.len(), 2);
            assert!(bookmarks.contains(&"b".to_string()));
            assert!(bookmarks.contains(&"c".to_string()));
            assert!(!bookmarks.contains(&"a".to_string()));
        }
    }

    #[test]
    fn test_push_select_none_clears_all() {
        let tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["a", "b"])]);
        let mut state = TestState::new(tree);
        state.reduce(Action::GitPush);

        state.reduce(Action::PushSelectNone);

        let ModeState::PushSelect(push_state) = &state.mode else {
            panic!("expected PushSelect mode");
        };
        assert!(push_state.selected.is_empty());
    }

    #[test]
    fn test_push_select_all_selects_all() {
        let tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["a", "b"])]);
        let mut state = TestState::new(tree);
        state.reduce(Action::GitPush);
        state.reduce(Action::PushSelectNone);
        state.reduce(Action::PushSelectAll);

        let ModeState::PushSelect(push_state) = &state.mode else {
            panic!("expected PushSelect mode");
        };
        assert_eq!(push_state.selected.len(), 2);
    }

    #[test]
    fn test_exit_push_select_returns_to_normal() {
        let tree = make_tree(vec![make_node_with_bookmarks("rev0", 0, &["a", "b"])]);
        let mut state = TestState::new(tree);
        state.reduce(Action::GitPush);

        state.reduce(Action::ExitPushSelect);

        assert!(matches!(state.mode, ModeState::Normal));
    }

    #[test]
    fn test_undo_produces_run_undo_and_refresh() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::Undo);

        assert_eq!(effects.len(), 2);
        assert!(matches!(effects[0], Effect::RunUndo));
        assert!(matches!(effects[1], Effect::RefreshTree));
    }

    #[test]
    fn test_edit_working_copy_with_empty_tree_shows_error() {
        let tree = make_tree(vec![]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::EditWorkingCopy);

        assert!(effects.iter().any(|e| matches!(
            e,
            Effect::SetStatus {
                kind: MessageKind::Error,
                ..
            }
        )));
        // should NOT produce RunEdit with empty rev
        assert!(!effects.iter().any(|e| matches!(e, Effect::RunEdit { .. })));
    }

    #[test]
    fn test_create_new_commit_with_empty_tree_shows_error() {
        let tree = make_tree(vec![]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::CreateNewCommit);

        assert!(effects.iter().any(|e| matches!(
            e,
            Effect::SetStatus {
                kind: MessageKind::Error,
                ..
            }
        )));
        assert!(!effects.iter().any(|e| matches!(e, Effect::RunNew { .. })));
    }

    #[test]
    fn test_edit_description_with_empty_tree_shows_error() {
        let tree = make_tree(vec![]);
        let mut state = TestState::new(tree);

        let effects = state.reduce(Action::EditDescription);

        assert!(effects.iter().any(|e| matches!(
            e,
            Effect::SetStatus {
                kind: MessageKind::Error,
                ..
            }
        )));
        assert!(state.pending_operation.is_none());
    }
}
