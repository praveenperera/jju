mod clipboard;
mod confirm;
mod conflicts;
mod diff;
mod rebase;
mod squash;

use super::{Action, Effect, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::ConflictsState;

pub(super) fn handle(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action {
        Action::EnterDiffView => diff::enter_diff_view(ctx),
        Action::ExitDiffView => *ctx.mode = ModeState::Normal,
        Action::EnterConfirmStackSync => confirm::enter_stack_sync(ctx),
        Action::EnterConfirmAbandon => confirm::enter_abandon(ctx),
        Action::EnterConfirmRebaseOntoTrunk(rebase_type) => {
            confirm::enter_rebase_onto_trunk(ctx, rebase_type);
        }
        Action::ConfirmYes => confirm::confirm_yes(ctx),
        Action::ConfirmNo => *ctx.mode = ModeState::Normal,
        Action::EnterRebaseMode(rebase_type) => rebase::enter(ctx, rebase_type),
        Action::ExitRebaseMode => *ctx.mode = ModeState::Normal,
        Action::MoveRebaseDestUp => rebase::move_dest_up(ctx),
        Action::MoveRebaseDestDown => rebase::move_dest_down(ctx),
        Action::ToggleRebaseBranches => rebase::toggle_branches(ctx),
        Action::ExecuteRebase => rebase::execute(ctx),
        Action::EnterSquashMode => squash::enter(ctx),
        Action::ExitSquashMode => *ctx.mode = ModeState::Normal,
        Action::MoveSquashDestUp => squash::move_dest_up(ctx),
        Action::MoveSquashDestDown => squash::move_dest_down(ctx),
        Action::ExecuteSquash => squash::execute(ctx),
        Action::ScrollDiffUp(amount) => diff::scroll_up(ctx, amount),
        Action::ScrollDiffDown(amount) => diff::scroll_down(ctx, amount),
        Action::ScrollDiffTop => diff::scroll_top(ctx),
        Action::ScrollDiffBottom => diff::scroll_bottom(ctx),
        Action::CopyBranchSelection(key) => clipboard::copy_branch_selection(ctx, key),
        Action::ExitClipboardMode => *ctx.mode = ModeState::Normal,
        Action::EnterConflicts => {
            *ctx.mode = ModeState::Conflicts(ConflictsState::default());
            ctx.effects.push(Effect::LoadConflictFiles);
        }
        Action::ExitConflicts => *ctx.mode = ModeState::Normal,
        Action::ConflictsUp => conflicts::move_up(ctx),
        Action::ConflictsDown => conflicts::move_down(ctx),
        Action::ConflictsJump => *ctx.mode = ModeState::Normal,
        Action::StartResolveFromConflicts => conflicts::start_resolve(ctx),
        _ => unreachable!("unsupported mode action: {action:?}"),
    }
}
