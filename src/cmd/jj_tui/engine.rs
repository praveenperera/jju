//! Engine for jj_tui
//!
//! The engine is a pure function that processes actions and produces effects.
//! It mutates state but performs no IO.

use super::action::Action;
use super::effect::Effect;
use super::handlers;
use super::state::{
    BookmarkInputState, BookmarkPickerState, BookmarkSelectAction, ConfirmAction, ConfirmState,
    ConflictsState, DiffState, MessageKind, ModeState, MovingBookmarkState, PendingOperation,
    PendingSquash, PushSelectState, RebaseState, RebaseType, SquashState,
};
use super::tree::TreeState;
use crate::jj_lib_helpers::JjRepo;
use ahash::{HashSet, HashSetExt};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Process an action and produce effects
/// Returns effects to be executed by the runner
#[allow(clippy::too_many_arguments)]
pub fn reduce(
    tree: &mut TreeState,
    mode: &mut ModeState,
    should_quit: &mut bool,
    split_view: &mut bool,
    pending_key: &mut Option<char>,
    pending_operation: &mut Option<PendingOperation>,
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
    action: Action,
    _viewport_height: usize,
) -> Vec<Effect> {
    let mut effects = Vec::new();

    match action {
        // Lifecycle
        Action::Quit => *should_quit = true,
        Action::RefreshTree => effects.push(Effect::RefreshTree),
        Action::Noop => {}

        // Pending key management
        Action::SetPendingKey(c) => *pending_key = Some(c),
        Action::ClearPendingKey => *pending_key = None,

        // Navigation
        Action::MoveCursorUp => {
            tree.move_cursor_up();
            // in selection mode, also extend selection
            if matches!(mode, ModeState::Selecting) {
                extend_selection_to_cursor(tree);
            }
        }
        Action::MoveCursorDown => {
            tree.move_cursor_down();
            // in selection mode, also extend selection
            if matches!(mode, ModeState::Selecting) {
                extend_selection_to_cursor(tree);
            }
        }
        Action::MoveCursorTop => tree.move_cursor_top(),
        Action::MoveCursorBottom => tree.move_cursor_bottom(),
        Action::JumpToWorkingCopy => tree.jump_to_working_copy(),
        Action::PageUp(amount) => tree.page_up(amount),
        Action::PageDown(amount) => tree.page_down(amount),
        Action::CenterCursor(vh) => {
            if vh > 0 {
                let half = vh / 2;
                tree.scroll_offset = tree.cursor.saturating_sub(half);
            }
        }

        // Focus/View
        Action::ToggleFocus => tree.toggle_focus(),
        Action::Unfocus => tree.unfocus(),
        Action::ToggleExpanded => tree.toggle_expanded(),
        Action::ToggleFullMode => tree.toggle_full_mode(),
        Action::ToggleSplitView => *split_view = !*split_view,

        // Mode transitions
        Action::EnterHelp => *mode = ModeState::Help,
        Action::ExitHelp => *mode = ModeState::Normal,

        Action::EnterDiffView => {
            let rev = current_rev(tree);
            if let Ok(diff_output) = super::commands::diff::get_diff(&rev) {
                let lines = handlers::diff::parse_diff(&diff_output, syntax_set, theme_set);
                *mode = ModeState::ViewingDiff(DiffState {
                    lines,
                    scroll_offset: 0,
                    rev,
                });
            }
        }
        Action::ExitDiffView => *mode = ModeState::Normal,

        Action::EnterConfirmAbandon => {
            let revs = get_revs_for_action(tree);
            // check for working copy in selection
            for rev in &revs {
                if tree
                    .nodes
                    .iter()
                    .any(|n| n.change_id == *rev && n.is_working_copy)
                {
                    effects.push(Effect::SetStatus {
                        text: "Cannot abandon working copy".to_string(),
                        kind: MessageKind::Error,
                    });
                    return effects;
                }
            }

            let message = if revs.len() == 1 {
                format!("Abandon revision {}?", revs[0])
            } else {
                format!("Abandon {} revisions?", revs.len())
            };

            *mode = ModeState::Confirming(ConfirmState {
                action: ConfirmAction::Abandon,
                message,
                revs,
            });
        }

        Action::EnterConfirmRebaseOntoTrunk(rebase_type) => {
            let source = current_rev(tree);
            if source.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            let short_rev = &source[..8.min(source.len())];
            let message = match rebase_type {
                RebaseType::Single => format!("Rebase {} onto trunk?", short_rev),
                RebaseType::WithDescendants => {
                    format!("Rebase {} and descendants onto trunk?", short_rev)
                }
            };

            let mode_flag = match rebase_type {
                RebaseType::Single => "-r",
                RebaseType::WithDescendants => "-s",
            };
            let cmd_preview = format!(
                "jj rebase {} {} -d trunk() --skip-emptied",
                mode_flag, short_rev
            );

            *mode = ModeState::Confirming(ConfirmState {
                action: ConfirmAction::RebaseOntoTrunk(rebase_type),
                message,
                revs: vec![cmd_preview],
            });
        }

        Action::EnterConfirmMoveBookmarkBackwards {
            ref bookmark_name,
            ref dest_rev,
            ref op_before,
        } => {
            let short_dest = &dest_rev[..8.min(dest_rev.len())];
            *mode = ModeState::Confirming(ConfirmState {
                action: ConfirmAction::MoveBookmarkBackwards {
                    bookmark_name: bookmark_name.clone(),
                    dest_rev: dest_rev.clone(),
                    op_before: op_before.clone(),
                },
                message: format!(
                    "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                    bookmark_name, short_dest
                ),
                revs: vec![],
            });
        }

        Action::ConfirmYes => {
            let ModeState::Confirming(state) = std::mem::replace(mode, ModeState::Normal) else {
                return effects;
            };

            match state.action {
                ConfirmAction::Abandon => {
                    let revset = state.revs.join(" | ");
                    effects.push(Effect::SaveOperationForUndo);
                    effects.push(Effect::RunAbandon { revset });
                    effects.push(Effect::RefreshTree);
                }
                ConfirmAction::RebaseOntoTrunk(rebase_type) => {
                    let source = current_rev(tree);
                    effects.push(Effect::SaveOperationForUndo);
                    effects.push(Effect::RunRebaseOntoTrunk {
                        source,
                        rebase_type,
                    });
                    effects.push(Effect::RefreshTree);
                }
                ConfirmAction::MoveBookmarkBackwards {
                    bookmark_name,
                    dest_rev,
                    op_before: _,
                } => {
                    effects.push(Effect::SaveOperationForUndo);
                    effects.push(Effect::RunBookmarkSetBackwards {
                        name: bookmark_name,
                        rev: dest_rev,
                    });
                    effects.push(Effect::RefreshTree);
                }
            }
            tree.clear_selection();
        }

        Action::ConfirmNo => *mode = ModeState::Normal,

        Action::EnterSelecting => {
            tree.selection_anchor = Some(tree.cursor);
            tree.selected.insert(tree.cursor);
            *mode = ModeState::Selecting;
        }

        Action::ExitSelecting => {
            *mode = ModeState::Normal;
            tree.selection_anchor = None;
        }

        Action::EnterRebaseMode(rebase_type) => {
            let source_rev = current_rev(tree);
            if source_rev.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            let current = tree.cursor;

            // create initial state
            let state = RebaseState {
                source_rev: source_rev.clone(),
                rebase_type,
                dest_cursor: current,
                allow_branches: false,
                op_before: String::new(), // will be filled by runner
            };

            // temporarily set mode to compute moving indices
            *mode = ModeState::Rebasing(state.clone());

            // compute moving indices and find initial position
            let moving = compute_moving_indices(tree, mode);
            let max = tree.visible_count();

            // get source's structural depth
            let source_struct_depth = tree
                .visible_entries
                .get(current)
                .map(|e| tree.nodes[e.node_index].depth)
                .unwrap_or(0);

            // find source's parent
            let mut initial_cursor = current.saturating_sub(1);
            while initial_cursor > 0 {
                let entry = &tree.visible_entries[initial_cursor];
                let node = &tree.nodes[entry.node_index];
                if node.depth < source_struct_depth && !moving.contains(&initial_cursor) {
                    break;
                }
                initial_cursor -= 1;
            }

            // verify we found a valid non-moving entry
            if moving.contains(&initial_cursor) || initial_cursor >= max {
                initial_cursor = 0;
                while initial_cursor < max && moving.contains(&initial_cursor) {
                    initial_cursor += 1;
                }
            }

            if let ModeState::Rebasing(s) = mode {
                s.dest_cursor = initial_cursor;
            }

            effects.push(Effect::SaveOperationForUndo);
        }

        Action::ExitRebaseMode => *mode = ModeState::Normal,

        Action::EnterSquashMode => {
            let source_rev = current_rev(tree);
            if source_rev.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            let current = tree.cursor;
            let source_struct_depth = tree
                .visible_entries
                .get(current)
                .map(|e| tree.nodes[e.node_index].depth)
                .unwrap_or(0);

            // find source's parent
            let mut initial_cursor = current.saturating_sub(1);
            while initial_cursor > 0 {
                let entry = &tree.visible_entries[initial_cursor];
                let node = &tree.nodes[entry.node_index];
                if node.depth < source_struct_depth {
                    break;
                }
                initial_cursor -= 1;
            }

            *mode = ModeState::Squashing(SquashState {
                source_rev,
                dest_cursor: initial_cursor,
                op_before: String::new(), // will be filled by runner
            });

            effects.push(Effect::SaveOperationForUndo);
        }

        Action::ExitSquashMode => *mode = ModeState::Normal,

        Action::EnterMoveBookmarkMode => {
            let Some(node) = tree.current_node() else {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            };

            let target_rev = node.change_id.clone();

            // Always enter picker mode so you can move any bookmark onto this revision.
            // Bookmarks already on this revision are pinned at the top.
            if let Ok(jj_repo) = JjRepo::load(None) {
                let all_bookmarks = jj_repo.all_local_bookmarks();
                if all_bookmarks.is_empty() {
                    effects.push(Effect::SetStatus {
                        text: "No bookmarks in repository".to_string(),
                        kind: MessageKind::Warning,
                    });
                    return effects;
                }

                let pinned = node.bookmark_names();
                let all_bookmarks = build_move_bookmark_picker_list(all_bookmarks, pinned, tree);

                *mode = ModeState::BookmarkPicker(BookmarkPickerState {
                    all_bookmarks,
                    filter: String::new(),
                    filter_cursor: 0,
                    selected_index: 0,
                    target_rev,
                    action: BookmarkSelectAction::Move,
                });
            }
        }

        Action::EnterCreateBookmark => {
            let rev = current_rev(tree);
            if rev.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            *mode = ModeState::BookmarkInput(BookmarkInputState {
                name: String::new(),
                cursor: 0,
                target_rev: rev.clone(),
                deleting: false,
            });

            let short_rev = &rev[..8.min(rev.len())];
            effects.push(Effect::SetStatus {
                text: format!("Creating bookmark at {}", short_rev),
                kind: MessageKind::Success,
            });
        }

        Action::EnterBookmarkPicker(action) => {
            if let Ok(jj_repo) = JjRepo::load(None) {
                let mut all_bookmarks = jj_repo.all_local_bookmarks();

                if all_bookmarks.is_empty() {
                    effects.push(Effect::SetStatus {
                        text: "No bookmarks in repository".to_string(),
                        kind: MessageKind::Warning,
                    });
                    return effects;
                }

                // for delete action, prioritize current commit's bookmarks
                if action == BookmarkSelectAction::Delete {
                    let current_bookmarks: Vec<String> = tree
                        .current_node()
                        .map(|n| n.bookmark_names())
                        .unwrap_or_default();

                    if !current_bookmarks.is_empty() {
                        all_bookmarks.retain(|b| !current_bookmarks.contains(b));
                        let mut reordered = current_bookmarks;
                        reordered.extend(all_bookmarks);
                        all_bookmarks = reordered;
                    }
                }

                let target_rev = tree
                    .current_node()
                    .map(|n| n.change_id.clone())
                    .unwrap_or_default();

                *mode = ModeState::BookmarkPicker(BookmarkPickerState {
                    all_bookmarks,
                    filter: String::new(),
                    filter_cursor: 0,
                    selected_index: 0,
                    target_rev,
                    action,
                });
            }
        }

        Action::ExitBookmarkMode => *mode = ModeState::Normal,

        // Selection
        Action::ToggleSelection => tree.toggle_selected(tree.cursor),
        Action::ExtendSelectionToCursor => extend_selection_to_cursor(tree),
        Action::ClearSelection => tree.clear_selection(),

        // Rebase mode navigation
        Action::MoveRebaseDestUp => {
            let moving = compute_moving_indices(tree, mode);
            if let ModeState::Rebasing(state) = mode {
                let mut next = state.dest_cursor.saturating_sub(1);
                while next > 0 && moving.contains(&next) {
                    next -= 1;
                }
                if !moving.contains(&next) {
                    state.dest_cursor = next;
                }
            }
        }

        Action::MoveRebaseDestDown => {
            let moving = compute_moving_indices(tree, mode);
            let max = tree.visible_count();
            if let ModeState::Rebasing(state) = mode {
                let mut next = state.dest_cursor + 1;
                while next < max && moving.contains(&next) {
                    next += 1;
                }
                if next < max {
                    state.dest_cursor = next;
                }
            }
        }

        Action::ToggleRebaseBranches => {
            if let ModeState::Rebasing(state) = mode {
                state.allow_branches = !state.allow_branches;
            }
        }

        Action::ExecuteRebase => {
            let ModeState::Rebasing(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            let Some(dest) = get_rev_at_cursor(tree, state.dest_cursor) else {
                effects.push(Effect::SetStatus {
                    text: "Invalid destination".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            };

            if state.source_rev == dest {
                effects.push(Effect::SetStatus {
                    text: "Cannot rebase onto self".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            effects.push(Effect::RunRebase {
                source: state.source_rev.clone(),
                dest,
                rebase_type: state.rebase_type,
                allow_branches: state.allow_branches,
            });
            effects.push(Effect::RefreshTree);
            *mode = ModeState::Normal;
        }

        // Squash mode navigation
        Action::MoveSquashDestUp => {
            if let ModeState::Squashing(state) = mode
                && state.dest_cursor > 0
            {
                state.dest_cursor -= 1;
            }
        }

        Action::MoveSquashDestDown => {
            if let ModeState::Squashing(state) = mode {
                let max = tree.visible_count().saturating_sub(1);
                if state.dest_cursor < max {
                    state.dest_cursor += 1;
                }
            }
        }

        Action::ExecuteSquash => {
            let ModeState::Squashing(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            let Some(target) = get_rev_at_cursor(tree, state.dest_cursor) else {
                effects.push(Effect::SetStatus {
                    text: "Invalid target".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            };

            if state.source_rev == target {
                effects.push(Effect::SetStatus {
                    text: "Cannot squash into self".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            // set pending squash - requires terminal restore for editor
            *pending_operation = Some(PendingOperation::Squash(PendingSquash {
                source_rev: state.source_rev.clone(),
                target_rev: target,
                op_before: state.op_before.clone(),
            }));
            *mode = ModeState::Normal;
        }

        // Bookmark modes navigation
        Action::MoveBookmarkDestUp => {
            if let ModeState::MovingBookmark(state) = mode
                && state.dest_cursor > 0
            {
                state.dest_cursor -= 1;
            }
        }

        Action::MoveBookmarkDestDown => {
            if let ModeState::MovingBookmark(state) = mode {
                let max = tree.visible_count().saturating_sub(1);
                if state.dest_cursor < max {
                    state.dest_cursor += 1;
                }
            }
        }

        Action::ExecuteBookmarkMove => {
            let ModeState::MovingBookmark(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            let Some(dest) = get_rev_at_cursor(tree, state.dest_cursor) else {
                effects.push(Effect::SetStatus {
                    text: "Invalid destination".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            };

            let name = state.bookmark_name.clone();

            // check if this move would be backwards
            if is_bookmark_move_backwards(tree, &name, &dest) {
                let short_dest = &dest[..8.min(dest.len())];
                *mode = ModeState::Confirming(ConfirmState {
                    action: ConfirmAction::MoveBookmarkBackwards {
                        bookmark_name: name.clone(),
                        dest_rev: dest.clone(),
                        op_before: state.op_before.clone(),
                    },
                    message: format!(
                        "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                        name, short_dest
                    ),
                    revs: vec![],
                });
                return effects;
            }

            // normal forward move
            effects.push(Effect::RunBookmarkSet { name, rev: dest });
            effects.push(Effect::RefreshTree);
            *mode = ModeState::Normal;
        }

        Action::SelectBookmarkUp => {
            if let ModeState::BookmarkSelect(state) = mode
                && state.selected_index > 0
            {
                state.selected_index -= 1;
            }
        }

        Action::SelectBookmarkDown => {
            if let ModeState::BookmarkSelect(state) = mode
                && state.selected_index < state.bookmarks.len().saturating_sub(1)
            {
                state.selected_index += 1;
            }
        }

        Action::ConfirmBookmarkSelect => {
            let ModeState::BookmarkSelect(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            let bookmark = state.bookmarks[state.selected_index].clone();

            match state.action {
                BookmarkSelectAction::Move => {
                    *mode = ModeState::MovingBookmark(MovingBookmarkState {
                        bookmark_name: bookmark,
                        dest_cursor: tree.cursor,
                        op_before: String::new(), // will be filled by runner
                    });
                    effects.push(Effect::SaveOperationForUndo);
                }
                BookmarkSelectAction::Delete => {
                    effects.push(Effect::SaveOperationForUndo);
                    effects.push(Effect::RunBookmarkDelete { name: bookmark });
                    effects.push(Effect::RefreshTree);
                    *mode = ModeState::Normal;
                }
            }
        }

        Action::BookmarkPickerUp => {
            if let ModeState::BookmarkPicker(state) = mode
                && state.selected_index > 0
            {
                state.selected_index -= 1;
            }
        }

        Action::BookmarkPickerDown => {
            if let ModeState::BookmarkPicker(state) = mode {
                let filtered_count = state.filtered_bookmarks().len();
                if state.selected_index < filtered_count.saturating_sub(1) {
                    state.selected_index += 1;
                }
            }
        }

        Action::BookmarkFilterChar(c) => {
            if let ModeState::BookmarkPicker(state) = mode {
                state.filter.insert(state.filter_cursor, c);
                state.filter_cursor += c.len_utf8();
                state.selected_index = 0;
            }
        }

        Action::BookmarkFilterBackspace => {
            if let ModeState::BookmarkPicker(state) = mode
                && state.filter_cursor > 0
            {
                let prev = state.filter[..state.filter_cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                state.filter.remove(prev);
                state.filter_cursor = prev;
                state.selected_index = 0;
            }
        }

        Action::ConfirmBookmarkPicker => {
            let ModeState::BookmarkPicker(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            let filtered = state.filtered_bookmarks();
            let Some(bookmark) = filtered.get(state.selected_index) else {
                effects.push(Effect::SetStatus {
                    text: "No bookmark selected".to_string(),
                    kind: MessageKind::Warning,
                });
                *mode = ModeState::Normal;
                return effects;
            };

            let bookmark_name = (*bookmark).clone();
            let target_rev = state.target_rev.clone();
            let action = state.action;

            match action {
                BookmarkSelectAction::Move => {
                    // If the selected bookmark is already on this revision, enter the
                    // destination-selection flow to move it elsewhere (keep existing behavior).
                    if bookmark_is_on_rev(tree, &bookmark_name, &target_rev) {
                        *mode = ModeState::MovingBookmark(MovingBookmarkState {
                            bookmark_name,
                            dest_cursor: tree.cursor,
                            op_before: String::new(), // will be filled by runner
                        });
                        effects.push(Effect::SaveOperationForUndo);
                        return effects;
                    }

                    if is_bookmark_move_backwards(tree, &bookmark_name, &target_rev) {
                        let short_dest = &target_rev[..8.min(target_rev.len())];
                        *mode = ModeState::Confirming(ConfirmState {
                            action: ConfirmAction::MoveBookmarkBackwards {
                                bookmark_name: bookmark_name.clone(),
                                dest_rev: target_rev.clone(),
                                op_before: String::new(), // will be filled by runner
                            },
                            message: format!(
                                "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                                bookmark_name, short_dest
                            ),
                            revs: vec![],
                        });
                        effects.push(Effect::SaveOperationForUndo);
                    } else {
                        effects.push(Effect::SaveOperationForUndo);
                        effects.push(Effect::RunBookmarkSet {
                            name: bookmark_name,
                            rev: target_rev,
                        });
                        effects.push(Effect::RefreshTree);
                        *mode = ModeState::Normal;
                    }
                }
                BookmarkSelectAction::Delete => {
                    effects.push(Effect::SaveOperationForUndo);
                    effects.push(Effect::RunBookmarkDelete {
                        name: bookmark_name,
                    });
                    effects.push(Effect::RefreshTree);
                    *mode = ModeState::Normal;
                }
            }
        }

        // Bookmark input
        Action::BookmarkInputChar(c) => {
            if let ModeState::BookmarkInput(state) = mode {
                state.name.insert(state.cursor, c);
                state.cursor += c.len_utf8();
            }
        }

        Action::BookmarkInputBackspace => {
            if let ModeState::BookmarkInput(state) = mode
                && state.cursor > 0
            {
                let prev = state.name[..state.cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                state.name.remove(prev);
                state.cursor = prev;
            }
        }

        Action::BookmarkInputDelete => {
            if let ModeState::BookmarkInput(state) = mode
                && state.cursor < state.name.len()
            {
                state.name.remove(state.cursor);
            }
        }

        Action::BookmarkInputCursorLeft => {
            if let ModeState::BookmarkInput(state) = mode
                && state.cursor > 0
            {
                state.cursor = state.name[..state.cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
            }
        }

        Action::BookmarkInputCursorRight => {
            if let ModeState::BookmarkInput(state) = mode
                && state.cursor < state.name.len()
            {
                state.cursor = state.name[state.cursor..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| state.cursor + i)
                    .unwrap_or(state.name.len());
            }
        }

        Action::ConfirmBookmarkInput => {
            let ModeState::BookmarkInput(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            if state.name.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "Bookmark name cannot be empty".to_string(),
                    kind: MessageKind::Error,
                });
                *mode = ModeState::Normal;
                return effects;
            }

            let name = state.name.clone();
            let target = state.target_rev.clone();
            let deleting = state.deleting;

            effects.push(Effect::SaveOperationForUndo);
            if deleting {
                effects.push(Effect::RunBookmarkDelete { name });
            } else {
                effects.push(Effect::RunBookmarkSet { name, rev: target });
            }
            effects.push(Effect::RefreshTree);
            *mode = ModeState::Normal;
        }

        // Diff view scrolling
        Action::ScrollDiffUp(amount) => {
            if let ModeState::ViewingDiff(state) = mode {
                state.scroll_offset = state.scroll_offset.saturating_sub(amount);
            }
            *pending_key = None;
        }

        Action::ScrollDiffDown(amount) => {
            if let ModeState::ViewingDiff(state) = mode {
                state.scroll_offset = state.scroll_offset.saturating_add(amount);
            }
            *pending_key = None;
        }

        Action::ScrollDiffTop => {
            if let ModeState::ViewingDiff(state) = mode {
                state.scroll_offset = 0;
            }
            *pending_key = None;
        }

        Action::ScrollDiffBottom => {
            if let ModeState::ViewingDiff(state) = mode {
                state.scroll_offset = state.lines.len().saturating_sub(1);
            }
            *pending_key = None;
        }

        // Commands
        Action::EditWorkingCopy => {
            let rev = current_rev(tree);
            if let Some(node) = tree.current_node()
                && node.is_working_copy
            {
                effects.push(Effect::SetStatus {
                    text: "Already editing this revision".to_string(),
                    kind: MessageKind::Warning,
                });
                return effects;
            }
            effects.push(Effect::RunEdit { rev });
            effects.push(Effect::RefreshTree);
        }

        Action::CreateNewCommit => {
            let rev = current_rev(tree);
            effects.push(Effect::RunNew { rev });
            effects.push(Effect::RefreshTree);
        }

        Action::CommitWorkingCopy => {
            if let Some(node) = tree.current_node()
                && !node.is_working_copy
            {
                effects.push(Effect::SetStatus {
                    text: "Can only commit from working copy (@)".to_string(),
                    kind: MessageKind::Warning,
                });
                return effects;
            }
            if let Some(node) = tree.current_node() {
                let message = if node.description.is_empty() {
                    "(no description)".to_string()
                } else {
                    node.description.clone()
                };
                effects.push(Effect::RunCommit { message });
                effects.push(Effect::RefreshTree);
            }
        }

        Action::EditDescription => {
            *pending_operation = Some(PendingOperation::EditDescription {
                rev: current_rev(tree),
            });
        }

        Action::ExecuteAbandon { ref revs } => {
            let revset = revs.join(" | ");
            effects.push(Effect::SaveOperationForUndo);
            effects.push(Effect::RunAbandon { revset });
            effects.push(Effect::RefreshTree);
            tree.clear_selection();
        }

        Action::ExecuteRebaseOntoTrunk(rebase_type) => {
            let source = current_rev(tree);
            effects.push(Effect::SaveOperationForUndo);
            effects.push(Effect::RunRebaseOntoTrunk {
                source,
                rebase_type,
            });
            effects.push(Effect::RefreshTree);
        }

        Action::Undo => {
            effects.push(Effect::RunUndo {
                op_id: String::new(), // runner will use last_op
            });
            effects.push(Effect::RefreshTree);
        }

        Action::GitPush => {
            let Some(node) = tree.current_node() else {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            };

            if node.bookmarks.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No bookmark on this revision to push".to_string(),
                    kind: MessageKind::Warning,
                });
                return effects;
            }

            // single bookmark: push immediately
            if node.bookmarks.len() == 1 {
                let bookmark = node.bookmarks[0].name.clone();
                effects.push(Effect::RunGitPush { bookmark });
                effects.push(Effect::RefreshTree);
            } else {
                // multiple bookmarks: show multi-select picker with all pre-selected
                let all_bookmarks: Vec<String> =
                    node.bookmarks.iter().map(|b| b.name.clone()).collect();
                let selected: HashSet<usize> = (0..all_bookmarks.len()).collect();
                *mode = ModeState::PushSelect(PushSelectState {
                    all_bookmarks,
                    filter: String::new(),
                    filter_cursor: 0,
                    cursor_index: 0,
                    selected,
                });
            }
        }

        Action::GitPushAll => {
            effects.push(Effect::RunGitPushAll);
            effects.push(Effect::RefreshTree);
        }

        Action::GitFetch => {
            effects.push(Effect::RunGitFetch);
            effects.push(Effect::RefreshTree);
        }

        Action::GitImport => {
            effects.push(Effect::RunGitImport);
            effects.push(Effect::RefreshTree);
        }

        Action::GitExport => {
            effects.push(Effect::RunGitExport);
            effects.push(Effect::RefreshTree);
        }

        // Conflicts panel
        Action::EnterConflicts => {
            *mode = ModeState::Conflicts(ConflictsState::default());
            effects.push(Effect::LoadConflictFiles);
        }

        Action::ExitConflicts => {
            *mode = ModeState::Normal;
        }

        Action::ConflictsUp => {
            if let ModeState::Conflicts(state) = mode
                && state.selected_index > 0
            {
                state.selected_index -= 1;
            }
        }

        Action::ConflictsDown => {
            if let ModeState::Conflicts(state) = mode {
                let max = state.files.len().saturating_sub(1);
                if state.selected_index < max {
                    state.selected_index += 1;
                }
            }
        }

        Action::ConflictsJump => {
            // jump to the file in the tree if applicable - for now just exit
            *mode = ModeState::Normal;
        }

        Action::StartResolveFromConflicts => {
            if let ModeState::Conflicts(state) = mode
                && let Some(file) = state.files.get(state.selected_index).cloned()
            {
                *pending_operation = Some(PendingOperation::Resolve { file });
                *mode = ModeState::Normal;
            }
        }

        // Push select mode
        Action::PushSelectUp => {
            if let ModeState::PushSelect(state) = mode
                && state.cursor_index > 0
            {
                state.cursor_index -= 1;
            }
        }

        Action::PushSelectDown => {
            if let ModeState::PushSelect(state) = mode {
                let filtered_count = state.filtered_bookmarks().len();
                if state.cursor_index < filtered_count.saturating_sub(1) {
                    state.cursor_index += 1;
                }
            }
        }

        Action::PushSelectToggle => {
            if let ModeState::PushSelect(state) = mode {
                let filtered = state.filtered_bookmarks();
                if let Some(&(original_idx, _)) = filtered.get(state.cursor_index) {
                    if state.selected.contains(&original_idx) {
                        state.selected.remove(&original_idx);
                    } else {
                        state.selected.insert(original_idx);
                    }
                }
            }
        }

        Action::PushSelectAll => {
            if let ModeState::PushSelect(state) = mode {
                // select all in filtered view - collect indices first to avoid borrow conflict
                let filtered_indices: Vec<usize> = state
                    .filtered_bookmarks()
                    .into_iter()
                    .map(|(i, _)| i)
                    .collect();
                for idx in filtered_indices {
                    state.selected.insert(idx);
                }
            }
        }

        Action::PushSelectNone => {
            if let ModeState::PushSelect(state) = mode {
                // deselect all in filtered view - collect indices first to avoid borrow conflict
                let filtered_indices: Vec<usize> = state
                    .filtered_bookmarks()
                    .into_iter()
                    .map(|(i, _)| i)
                    .collect();
                for idx in filtered_indices {
                    state.selected.remove(&idx);
                }
            }
        }

        Action::PushSelectFilterChar(c) => {
            if let ModeState::PushSelect(state) = mode {
                state.filter.insert(state.filter_cursor, c);
                state.filter_cursor += c.len_utf8();
                state.cursor_index = 0;
            }
        }

        Action::PushSelectFilterBackspace => {
            if let ModeState::PushSelect(state) = mode
                && state.filter_cursor > 0
            {
                let prev = state.filter[..state.filter_cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                state.filter.remove(prev);
                state.filter_cursor = prev;
                state.cursor_index = 0;
            }
        }

        Action::PushSelectConfirm => {
            let ModeState::PushSelect(state) = &*mode else {
                *mode = ModeState::Normal;
                return effects;
            };

            if state.selected.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No bookmarks selected".to_string(),
                    kind: MessageKind::Warning,
                });
                *mode = ModeState::Normal;
                return effects;
            }

            let bookmarks: Vec<String> = state
                .selected
                .iter()
                .filter_map(|&idx| state.all_bookmarks.get(idx).cloned())
                .collect();

            effects.push(Effect::RunGitPushMultiple { bookmarks });
            effects.push(Effect::RefreshTree);
            *mode = ModeState::Normal;
        }

        Action::ExitPushSelect => {
            *mode = ModeState::Normal;
        }

        Action::ResolveDivergence => {
            let Some(node) = tree.current_node() else {
                effects.push(Effect::SetStatus {
                    text: "No revision selected".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            };

            if !node.is_divergent {
                effects.push(Effect::SetStatus {
                    text: "This revision is not divergent".to_string(),
                    kind: MessageKind::Warning,
                });
                return effects;
            }

            if node.divergent_versions.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "No divergent versions found".to_string(),
                    kind: MessageKind::Error,
                });
                return effects;
            }

            // find the local version (first one, typically newest/has working copy)
            // and the others to abandon
            let local_version = &node.divergent_versions[0];
            let abandon_ids: Vec<String> = node
                .divergent_versions
                .iter()
                .skip(1)
                .map(|v| v.commit_id.clone())
                .collect();

            if abandon_ids.is_empty() {
                effects.push(Effect::SetStatus {
                    text: "Only one version exists, nothing to resolve".to_string(),
                    kind: MessageKind::Warning,
                });
                return effects;
            }

            effects.push(Effect::SaveOperationForUndo);
            effects.push(Effect::RunResolveDivergence {
                keep_commit_id: local_version.commit_id.clone(),
                abandon_commit_ids: abandon_ids,
            });
            effects.push(Effect::RefreshTree);
        }
    }

    // clear pending key after processing most actions (except SetPendingKey)
    if !matches!(action, Action::SetPendingKey(_)) {
        *pending_key = None;
    }

    effects
}

// Helper functions

fn current_rev(tree: &TreeState) -> String {
    tree.current_node()
        .map(|n| n.change_id.clone())
        .unwrap_or_default()
}

fn get_revs_for_action(tree: &TreeState) -> Vec<String> {
    if tree.selected.is_empty() {
        vec![current_rev(tree)]
    } else {
        tree.selected
            .iter()
            .filter_map(|&idx| {
                tree.visible_entries
                    .get(idx)
                    .map(|e| tree.nodes[e.node_index].change_id.clone())
            })
            .collect()
    }
}

fn get_rev_at_cursor(tree: &TreeState, cursor: usize) -> Option<String> {
    tree.visible_entries
        .get(cursor)
        .map(|e| tree.nodes[e.node_index].change_id.clone())
}

fn extend_selection_to_cursor(tree: &mut TreeState) {
    if let Some(anchor) = tree.selection_anchor {
        tree.selected.clear();
        tree.select_range(anchor, tree.cursor);
    }
}

fn is_bookmark_move_backwards(tree: &TreeState, bookmark_name: &str, dest_rev: &str) -> bool {
    let Some(current_node) = tree.nodes.iter().find(|n| n.has_bookmark(bookmark_name)) else {
        return false; // new bookmark, not backwards
    };
    let current_change_id = &current_node.change_id;

    // check if dest is an ancestor of current position
    super::commands::is_ancestor(dest_rev, current_change_id).unwrap_or(false)
}

fn bookmark_is_on_rev(tree: &TreeState, bookmark_name: &str, rev: &str) -> bool {
    tree.nodes
        .iter()
        .any(|n| n.change_id == rev && n.has_bookmark(bookmark_name))
}

fn build_move_bookmark_picker_list(
    all_bookmarks: Vec<String>,
    pinned: Vec<String>,
    tree: &TreeState,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut pinned_unique = Vec::new();
    for name in pinned {
        if seen.insert(name.clone()) {
            pinned_unique.push(name);
        }
    }

    let pinned_set = seen;
    let mut rest: Vec<String> = all_bookmarks
        .into_iter()
        .filter(|b| !pinned_set.contains(b))
        .collect();
    sort_bookmarks_by_proximity(&mut rest, tree);

    let mut ordered = pinned_unique;
    ordered.extend(rest);
    ordered
}

/// Sort bookmarks by proximity to the current cursor position
/// Prefers bookmarks above the cursor (lower index = ancestors), so moving
/// bookmarks forward to newer commits is prioritized
fn sort_bookmarks_by_proximity(bookmarks: &mut [String], tree: &TreeState) {
    let bookmark_indices = tree.bookmark_to_visible_index();
    let cursor = tree.cursor;

    bookmarks.sort_by(|a, b| {
        let idx_a = bookmark_indices.get(a).copied();
        let idx_b = bookmark_indices.get(b).copied();

        // bookmarks not in visible tree go last
        match (idx_a, idx_b) {
            (None, None) => a.cmp(b),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(idx_a), Some(idx_b)) => {
                let above_a = idx_a < cursor;
                let above_b = idx_b < cursor;

                // prefer bookmarks above cursor
                match (above_a, above_b) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        // both above or both below: sort by distance, then alphabetically
                        let dist_a = idx_a.abs_diff(cursor);
                        let dist_b = idx_b.abs_diff(cursor);
                        dist_a.cmp(&dist_b).then_with(|| a.cmp(b))
                    }
                }
            }
        }
    });
}

/// Compute indices of entries that will move during rebase
pub fn compute_moving_indices(tree: &TreeState, mode: &ModeState) -> HashSet<usize> {
    let ModeState::Rebasing(state) = mode else {
        return HashSet::new();
    };

    let mut indices = HashSet::new();
    let mut in_source_tree = false;
    let mut source_struct_depth = 0usize;

    for (idx, entry) in tree.visible_entries.iter().enumerate() {
        let node = &tree.nodes[entry.node_index];

        if node.change_id == state.source_rev {
            indices.insert(idx);
            if state.rebase_type == RebaseType::WithDescendants {
                in_source_tree = true;
                source_struct_depth = node.depth;
            }
        } else if in_source_tree {
            if node.depth > source_struct_depth {
                indices.insert(idx);
            } else {
                break;
            }
        }
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::tree::{TreeNode, VisibleEntry};
    use ahash::HashMap;

    fn make_node(change_id: &str, depth: usize) -> TreeNode {
        TreeNode {
            change_id: change_id.to_string(),
            unique_prefix_len: 4,
            commit_id: format!("{change_id}000000"),
            unique_commit_prefix_len: 7,
            description: String::new(),
            full_description: String::new(),
            bookmarks: vec![],
            is_working_copy: false,
            has_conflicts: false,
            is_divergent: false,
            divergent_versions: vec![],
            parent_ids: vec![],
            depth,
            author_name: String::new(),
            author_email: String::new(),
            timestamp: String::new(),
        }
    }

    fn make_node_with_bookmarks(change_id: &str, depth: usize, bookmarks: &[&str]) -> TreeNode {
        let mut node = make_node(change_id, depth);
        node.bookmarks = bookmarks
            .iter()
            .map(|&name| crate::cmd::jj_tui::tree::BookmarkInfo {
                name: name.to_string(),
                is_diverged: false,
            })
            .collect();
        node
    }

    fn make_tree(nodes: Vec<TreeNode>) -> TreeState {
        let visible_entries: Vec<VisibleEntry> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| VisibleEntry {
                node_index: i,
                visual_depth: n.depth,
            })
            .collect();

        TreeState {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            expanded_entry: None,
            children_map: HashMap::default(),
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }

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
                &mut self.tree,
                &mut self.mode,
                &mut self.should_quit,
                &mut self.split_view,
                &mut self.pending_key,
                &mut self.pending_operation,
                &self.syntax_set,
                &self.theme_set,
                action,
                20, // viewport_height
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
        assert!(matches!(state.mode, ModeState::Help));

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

        let ordered = build_move_bookmark_picker_list(all_bookmarks, pinned, &tree);
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
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], Effect::RefreshTree));
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
}
