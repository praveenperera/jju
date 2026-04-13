use super::*;
use crate::cmd::jj_tui::state::{
    BookmarkPickerState, BookmarkSelectAction, ClipboardBranchSelectState,
};
use crate::cmd::jj_tui::test_support::{TestNodeKind, make_tree};
use crate::cmd::jj_tui::tree::{NeighborhoodExtent, TreeLoadScope};

struct TestState {
    tree: TreeState,
    mode: ModeState,
    should_quit: bool,
    split_view: bool,
    pending_key: Option<char>,
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
        TestNodeKind::Plain.make_node("aaaa", 0),
        TestNodeKind::Plain.make_node("bbbb", 1),
        TestNodeKind::Plain.make_node("cccc", 2),
    ]);
    let mut state = TestState::new(tree);

    assert_eq!(state.tree.view.cursor, 0);

    state.reduce(Action::MoveCursorDown);
    assert_eq!(state.tree.view.cursor, 1);

    state.reduce(Action::MoveCursorDown);
    assert_eq!(state.tree.view.cursor, 2);

    state.reduce(Action::MoveCursorDown);
    assert_eq!(state.tree.view.cursor, 2);

    state.reduce(Action::MoveCursorUp);
    assert_eq!(state.tree.view.cursor, 1);

    state.reduce(Action::MoveCursorTop);
    assert_eq!(state.tree.view.cursor, 0);

    state.reduce(Action::MoveCursorBottom);
    assert_eq!(state.tree.view.cursor, 2);
}

#[test]
fn test_mode_transitions() {
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
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
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
    let mut state = TestState::new(tree);

    assert!(!state.should_quit);
    state.reduce(Action::Quit);
    assert!(state.should_quit);
}

#[test]
fn test_pending_key() {
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
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
        TestNodeKind::Bookmarked(&["pin2", "pin1"]).make_node("rev0", 0),
        TestNodeKind::Bookmarked(&["near1"]).make_node("rev1", 1),
        TestNodeKind::Plain.make_node("rev2", 1),
        TestNodeKind::Bookmarked(&["near2"]).make_node("rev3", 1),
        TestNodeKind::Bookmarked(&["far"]).make_node("rev4", 1),
    ]);
    tree.view.cursor = 0;

    let all_bookmarks = vec![
        "near2".to_string(),
        "pin1".to_string(),
        "far".to_string(),
        "near1".to_string(),
        "pin2".to_string(),
    ];
    let pinned = tree.nodes()[0].bookmark_names();

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
fn test_copy_branch_single_bookmark() {
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["main"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::CopyBranch);

    assert!(matches!(
        effects.first(),
        Some(Effect::CopyToClipboard { value, .. }) if value == "main"
    ));
}

#[test]
fn test_copy_branch_multiple_bookmarks_enters_picker_mode() {
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["main", "feature"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::CopyBranch);

    assert!(effects.is_empty());
    assert!(matches!(
        state.mode,
        ModeState::ClipboardBranchSelect(ClipboardBranchSelectState { ref options, .. })
            if options.len() == 2 && options[0].key == 'a' && options[1].key == 'b'
    ));
}

#[test]
fn test_copy_branch_selection_copies_and_exits_mode() {
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["main", "feature"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);
    state.mode = ModeState::ClipboardBranchSelect(ClipboardBranchSelectState {
        target_rev: "rev0".to_string(),
        options: vec![
            crate::cmd::jj_tui::state::ClipboardBranchOption {
                key: 'a',
                branch: "main".to_string(),
            },
            crate::cmd::jj_tui::state::ClipboardBranchOption {
                key: 'b',
                branch: "feature".to_string(),
            },
        ],
    });

    let effects = state.reduce(Action::CopyBranchSelection('b'));

    assert!(matches!(
        effects.first(),
        Some(Effect::CopyToClipboard { value, .. }) if value == "feature"
    ));
    assert!(matches!(state.mode, ModeState::Normal));
}

#[test]
fn test_copy_commit_subject_uses_first_line() {
    let mut tree = make_tree(vec![TestNodeKind::Plain.make_node("rev0", 0)]);
    tree.snapshot.nodes[0].description = "subject line".to_string();
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::CopyCommitSubject);

    assert!(matches!(
        effects.first(),
        Some(Effect::CopyToClipboard { value, .. }) if value == "subject line"
    ));
}

#[test]
fn test_copy_selection_revset_uses_visible_order() {
    let tree = make_tree(vec![
        TestNodeKind::Plain.make_node("aaaa", 0),
        TestNodeKind::Plain.make_node("bbbb", 0),
        TestNodeKind::Plain.make_node("cccc", 0),
    ]);
    let mut state = TestState::new(tree);
    state.tree.view.selected.insert(2);
    state.tree.view.selected.insert(0);

    let effects = state.reduce(Action::CopySelectionRevset);

    assert!(matches!(
        effects.first(),
        Some(Effect::CopyToClipboard { value, .. }) if value == "aaaa | cccc"
    ));
}

#[test]
fn test_confirm_bookmark_picker_move_already_here_enters_move_away_flow() {
    let mut tree = make_tree(vec![TestNodeKind::Bookmarked(&["a"]).make_node("rev0", 0)]);
    tree.view.cursor = 0;

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
            .any(|effect| matches!(effect, Effect::SaveOperationForUndo))
    );
    assert!(!effects.iter().any(|effect| matches!(
        effect,
        Effect::RunBookmarkSet { .. } | Effect::RunBookmarkSetBackwards { .. }
    )));

    assert!(matches!(
        state.mode,
        ModeState::MovingBookmark(ref state)
            if state.bookmark_name == "a" && state.dest_cursor == 0
    ));
}

#[test]
fn test_selection_toggle() {
    let tree = make_tree(vec![
        TestNodeKind::Plain.make_node("aaaa", 0),
        TestNodeKind::Plain.make_node("bbbb", 1),
    ]);
    let mut state = TestState::new(tree);

    assert!(state.tree.view.selected.is_empty());

    state.reduce(Action::ToggleSelection);
    assert!(state.tree.view.selected.contains(&0));

    state.reduce(Action::ToggleSelection);
    assert!(!state.tree.view.selected.contains(&0));

    state.tree.view.cursor = 1;
    state.reduce(Action::ToggleSelection);
    assert!(state.tree.view.selected.contains(&1));

    state.reduce(Action::ClearSelection);
    assert!(state.tree.view.selected.is_empty());
}

#[test]
fn test_split_view_toggle() {
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
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
        TestNodeKind::Plain.make_node("aaaa", 0),
        TestNodeKind::Plain.make_node("bbbb", 1),
        TestNodeKind::Plain.make_node("cccc", 2),
    ]);
    let mut state = TestState::new(tree);

    state.reduce(Action::EnterSelecting);
    assert!(matches!(state.mode, ModeState::Selecting));
    assert!(state.tree.view.selected.contains(&0));
    assert_eq!(state.tree.view.selection_anchor, Some(0));

    state.reduce(Action::MoveCursorDown);
    assert!(state.tree.view.selected.contains(&0));
    assert!(state.tree.view.selected.contains(&1));

    state.reduce(Action::MoveCursorDown);
    assert!(state.tree.view.selected.contains(&0));
    assert!(state.tree.view.selected.contains(&1));
    assert!(state.tree.view.selected.contains(&2));
}

#[test]
fn test_page_navigation() {
    let tree = make_tree(vec![
        TestNodeKind::Plain.make_node("a", 0),
        TestNodeKind::Plain.make_node("b", 0),
        TestNodeKind::Plain.make_node("c", 0),
        TestNodeKind::Plain.make_node("d", 0),
        TestNodeKind::Plain.make_node("e", 0),
        TestNodeKind::Plain.make_node("f", 0),
        TestNodeKind::Plain.make_node("g", 0),
        TestNodeKind::Plain.make_node("h", 0),
        TestNodeKind::Plain.make_node("i", 0),
        TestNodeKind::Plain.make_node("j", 0),
    ]);
    let mut state = TestState::new(tree);

    state.reduce(Action::PageDown(5));
    assert_eq!(state.tree.view.cursor, 5);

    state.reduce(Action::PageUp(3));
    assert_eq!(state.tree.view.cursor, 2);

    state.reduce(Action::PageUp(10));
    assert_eq!(state.tree.view.cursor, 0);

    state.reduce(Action::PageDown(100));
    assert_eq!(state.tree.view.cursor, 9);
}

#[test]
fn test_noop_produces_no_effects() {
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::Noop);
    assert!(effects.is_empty());
}

#[test]
fn test_refresh_tree_produces_effect() {
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::RefreshTree);
    assert_eq!(effects.len(), 2);
    assert!(matches!(effects[0], Effect::RefreshTree));
    assert!(matches!(effects[1], Effect::SetStatus { .. }));
}

#[test]
fn test_toggle_neighborhood_switches_scope_and_refreshes() {
    let tree = make_tree(vec![
        TestNodeKind::Plain.make_node("aaaa", 0),
        TestNodeKind::Plain.make_node("bbbb", 1),
    ]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::ToggleNeighborhood);

    assert!(state.tree.is_neighborhood_mode());
    assert_eq!(state.tree.view.load_scope, TreeLoadScope::Neighborhood);
    assert_eq!(effects.len(), 1);
    assert!(matches!(effects[0], Effect::RefreshTree));

    let effects = state.reduce(Action::ToggleNeighborhood);

    assert!(!state.tree.is_neighborhood_mode());
    assert_eq!(state.tree.view.load_scope, TreeLoadScope::Stack);
    assert_eq!(effects.len(), 1);
    assert!(matches!(effects[0], Effect::RefreshTree));
}

#[test]
fn test_expand_neighborhood_refreshes_when_reaching_full_tree() {
    let nodes = (0..40)
        .map(|depth| TestNodeKind::Plain.make_node(&format!("n{depth:02}"), depth))
        .collect();
    let mut tree = make_tree(nodes);
    tree.view.cursor = 35;
    tree.enable_neighborhood();
    let mut state = TestState::new(tree);

    for _ in 0..6 {
        let effects = state.reduce(Action::ExpandNeighborhood);
        assert!(effects.is_empty());
    }

    let effects = state.reduce(Action::ExpandNeighborhood);

    assert_eq!(state.tree.view.load_scope, TreeLoadScope::Stack);
    assert_eq!(
        state
            .tree
            .neighborhood_state()
            .map(|state| state.extent.clone()),
        Some(NeighborhoodExtent::FullTree)
    );
    assert_eq!(effects.len(), 1);
    assert!(matches!(effects[0], Effect::RefreshTree));
}

#[test]
fn test_shrink_neighborhood_refreshes_when_leaving_full_tree() {
    let nodes = (0..40)
        .map(|depth| TestNodeKind::Plain.make_node(&format!("n{depth:02}"), depth))
        .collect();
    let mut tree = make_tree(nodes);
    tree.view.cursor = 35;
    tree.enable_neighborhood();
    for _ in 0..7 {
        let _ = tree.expand_neighborhood();
    }
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::ShrinkNeighborhood);

    assert_eq!(state.tree.view.load_scope, TreeLoadScope::Neighborhood);
    assert_eq!(
        state
            .tree
            .neighborhood_state()
            .map(|state| state.extent.clone()),
        Some(NeighborhoodExtent::Local(6))
    );
    assert_eq!(effects.len(), 1);
    assert!(matches!(effects[0], Effect::RefreshTree));
}

#[test]
fn test_git_push_single_bookmark_pushes_immediately() {
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["feature"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::GitPush);

    assert!(matches!(state.mode, ModeState::Normal));
    assert!(
        effects.iter().any(
            |effect| matches!(effect, Effect::RunGitPush { bookmark } if bookmark == "feature")
        )
    );
}

#[test]
fn test_git_push_multiple_bookmarks_enters_push_select_mode() {
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["feature-a", "feature-b"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::GitPush);

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
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["a", "b", "c"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);
    state.reduce(Action::GitPush);

    let ModeState::PushSelect(push_state) = &state.mode else {
        panic!("expected PushSelect mode");
    };
    assert_eq!(push_state.selected.len(), 3);

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
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["a", "b", "c"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);
    state.reduce(Action::GitPush);
    state.reduce(Action::PushSelectToggle);

    let effects = state.reduce(Action::PushSelectConfirm);

    assert!(matches!(state.mode, ModeState::Normal));
    let push_effect = effects
        .iter()
        .find(|effect| matches!(effect, Effect::RunGitPushMultiple { .. }));
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
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["a", "b"]).make_node("rev0", 0),
    ]);
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
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["a", "b"]).make_node("rev0", 0),
    ]);
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
    let tree = make_tree(vec![
        TestNodeKind::Bookmarked(&["a", "b"]).make_node("rev0", 0),
    ]);
    let mut state = TestState::new(tree);
    state.reduce(Action::GitPush);

    state.reduce(Action::ExitPushSelect);

    assert!(matches!(state.mode, ModeState::Normal));
}

#[test]
fn test_undo_produces_run_undo_and_refresh() {
    let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
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

    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::SetStatus {
            kind: MessageKind::Error,
            ..
        }
    )));
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::RunEdit { .. }))
    );
}

#[test]
fn test_create_new_commit_with_empty_tree_shows_error() {
    let tree = make_tree(vec![]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::CreateNewCommit);

    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::SetStatus {
            kind: MessageKind::Error,
            ..
        }
    )));
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::RunNew { .. }))
    );
}

#[test]
fn test_edit_description_with_empty_tree_shows_error() {
    let tree = make_tree(vec![]);
    let mut state = TestState::new(tree);

    let effects = state.reduce(Action::EditDescription);

    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::SetStatus {
            kind: MessageKind::Error,
            ..
        }
    )));
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::RunInteractive(_)))
    );
}
