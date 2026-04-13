mod move_flow;

use super::super::super::{ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::BookmarkSelectAction;

struct BookmarkPickerConfirmer<'a, 'b>(&'a mut ReduceCtx<'b>);

pub(super) fn confirm_bookmark_picker(ctx: &mut ReduceCtx<'_>) -> bool {
    let Some(state) = bookmark_picker_state(ctx) else {
        return false;
    };

    let selected_bookmark = state
        .filtered_bookmarks()
        .get(state.selected_index)
        .map(|bookmark| (*bookmark).clone());
    let target_rev = state.target_rev.clone();
    let action = state.action;
    let filter = state.filter.trim().to_string();

    let mut confirmer = BookmarkPickerConfirmer(ctx);

    let Some(bookmark_name) = selected_bookmark else {
        return confirmer.create_new_bookmark(action, &filter, &target_rev);
    };

    match action {
        BookmarkSelectAction::Move => {
            confirmer.confirm_move_bookmark_picker(bookmark_name, target_rev)
        }
        BookmarkSelectAction::Delete => confirmer.confirm_delete_bookmark_picker(bookmark_name),
        BookmarkSelectAction::CreatePR => {
            confirmer.confirm_create_pr_bookmark_picker(bookmark_name)
        }
    }

    true
}

fn bookmark_picker_state<'a>(
    ctx: &'a ReduceCtx<'_>,
) -> Option<&'a crate::cmd::jj_tui::state::BookmarkPickerState> {
    match &*ctx.mode {
        ModeState::BookmarkPicker(state) => Some(state),
        ModeState::Normal
        | ModeState::Selecting
        | ModeState::Rebasing(_)
        | ModeState::Squashing(_)
        | ModeState::ViewingDiff(_)
        | ModeState::Confirming(_)
        | ModeState::MovingBookmark(_)
        | ModeState::BookmarkSelect(_)
        | ModeState::PushSelect(_)
        | ModeState::Help(_)
        | ModeState::Conflicts(_) => None,
    }
}

impl BookmarkPickerConfirmer<'_, '_> {
    pub(super) fn create_new_bookmark(
        &mut self,
        action: BookmarkSelectAction,
        name: &str,
        target_rev: &str,
    ) -> bool {
        if action != BookmarkSelectAction::Move || name.is_empty() {
            return false;
        }

        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::SaveOperationForUndo);
        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::RunBookmarkSet {
                name: name.to_string(),
                rev: target_rev.to_string(),
            });
        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::RefreshTree);
        *self.0.mode = ModeState::Normal;
        true
    }

    pub(super) fn confirm_delete_bookmark_picker(&mut self, bookmark_name: String) {
        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::SaveOperationForUndo);
        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::RunBookmarkDelete {
                name: bookmark_name,
            });
        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::RefreshTree);
        *self.0.mode = ModeState::Normal;
    }

    pub(super) fn confirm_create_pr_bookmark_picker(&mut self, bookmark_name: String) {
        self.0
            .effects
            .push(crate::cmd::jj_tui::effect::Effect::RunCreatePR {
                bookmark: bookmark_name,
            });
        *self.0.mode = ModeState::Normal;
    }
}
