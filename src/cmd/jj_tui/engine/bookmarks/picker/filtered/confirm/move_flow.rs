use super::BookmarkPickerConfirmer;
use crate::cmd::jj_tui::effect::Effect;
use crate::cmd::jj_tui::engine::bookmarks::helpers::{
    bookmark_is_on_rev, is_bookmark_move_backwards,
};
use crate::cmd::jj_tui::state::{ConfirmAction, ConfirmState, ModeState, MovingBookmarkState};

impl BookmarkPickerConfirmer<'_, '_> {
    pub(super) fn confirm_move_bookmark_picker(
        &mut self,
        bookmark_name: String,
        target_rev: String,
    ) {
        if bookmark_is_on_rev(self.0.tree, &bookmark_name, &target_rev) {
            *self.0.mode = ModeState::MovingBookmark(MovingBookmarkState {
                bookmark_name,
                dest_cursor: self.0.tree.view.cursor,
            });
            self.0.effects.push(Effect::SaveOperationForUndo);
            return;
        }

        if is_bookmark_move_backwards(self.0.tree, &bookmark_name, &target_rev) {
            let short_dest = &target_rev[..8.min(target_rev.len())];
            *self.0.mode = ModeState::Confirming(ConfirmState {
                action: ConfirmAction::MoveBookmarkBackwards {
                    bookmark_name: bookmark_name.clone(),
                    dest_rev: target_rev.clone(),
                },
                message: format!(
                    "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                    bookmark_name, short_dest
                ),
                revs: vec![],
            });
            self.0.effects.push(Effect::SaveOperationForUndo);
            return;
        }

        self.0.effects.push(Effect::SaveOperationForUndo);
        self.0.effects.push(Effect::RunBookmarkSet {
            name: bookmark_name,
            rev: target_rev,
        });
        self.0.effects.push(Effect::RefreshTree);
        *self.0.mode = ModeState::Normal;
    }
}
