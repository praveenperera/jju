use super::{Action, Effect, MessageKind, ModeState, ReduceCtx, selection};
use crate::cmd::jj_tui::state::HelpState;

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::MoveCursorUp => {
            ctx.tree.move_cursor_up();
            if matches!(ctx.mode, ModeState::Selecting) {
                selection::extend_selection_to_cursor(ctx.tree);
            }
        }
        Action::MoveCursorDown => {
            ctx.tree.move_cursor_down();
            if matches!(ctx.mode, ModeState::Selecting) {
                selection::extend_selection_to_cursor(ctx.tree);
            }
        }
        Action::MoveCursorTop => ctx.tree.move_cursor_top(),
        Action::MoveCursorBottom => ctx.tree.move_cursor_bottom(),
        Action::JumpToWorkingCopy => ctx.tree.jump_to_working_copy(),
        Action::PageUp(amount) => ctx.tree.page_up(amount),
        Action::PageDown(amount) => ctx.tree.page_down(amount),
        Action::CenterCursor(viewport_height) => {
            if viewport_height > 0 {
                ctx.tree.view.scroll_offset =
                    ctx.tree.view.cursor.saturating_sub(viewport_height / 2);
            }
        }
        Action::ToggleFocus => ctx.tree.toggle_focus(),
        Action::ToggleNeighborhood => {
            ctx.tree.toggle_neighborhood();
            ctx.effects.push(Effect::RefreshTree);
        }
        Action::ExpandNeighborhood => {
            if !ctx.tree.expand_neighborhood() {
                ctx.set_status("Neighborhood already at maximum size", MessageKind::Warning);
            }
        }
        Action::ShrinkNeighborhood => {
            if !ctx.tree.shrink_neighborhood() {
                ctx.set_status("Neighborhood already at minimum size", MessageKind::Warning);
            }
        }
        Action::EnterNeighborhoodPath => {
            if !ctx.tree.enter_neighborhood_path() {
                ctx.set_status("No neighborhood path to open", MessageKind::Warning);
            }
        }
        Action::ExitNeighborhoodPath => {
            if !ctx.tree.exit_neighborhood_path() {
                ctx.set_status("Already at top neighborhood path", MessageKind::Warning);
            }
        }
        Action::Unfocus => ctx.tree.unfocus(),
        Action::ToggleExpanded => ctx.tree.toggle_expanded(),
        Action::ToggleFullMode => ctx.tree.toggle_full_mode(),
        Action::ToggleSplitView => *ctx.split_view = !*ctx.split_view,
        Action::EnterHelp => {
            *ctx.mode = ModeState::Help(HelpState { scroll_offset: 0 });
        }
        Action::ExitHelp => *ctx.mode = ModeState::Normal,
        Action::ScrollHelpUp(amount) => {
            if let ModeState::Help(state) = ctx.mode {
                state.scroll_offset = state.scroll_offset.saturating_sub(amount);
            }
        }
        Action::ScrollHelpDown(amount) => {
            if let ModeState::Help(state) = ctx.mode {
                state.scroll_offset = state.scroll_offset.saturating_add(amount);
            }
        }
        Action::EnterSelecting => {
            ctx.tree.view.selection_anchor = Some(ctx.tree.view.cursor);
            ctx.tree.view.selected.insert(ctx.tree.view.cursor);
            *ctx.mode = ModeState::Selecting;
        }
        Action::ExitSelecting => {
            *ctx.mode = ModeState::Normal;
            ctx.tree.view.selection_anchor = None;
        }
        Action::ToggleSelection => ctx.tree.toggle_selected(ctx.tree.view.cursor),
        Action::ClearSelection => ctx.tree.clear_selection(),
        Action::RefreshTree => {
            ctx.effects.push(Effect::RefreshTree);
            ctx.effects.push(Effect::SetStatus {
                text: "Refreshed".to_string(),
                kind: MessageKind::Success,
            });
        }
        _ => unreachable!("unsupported navigation action: {action:?}"),
    }
}
