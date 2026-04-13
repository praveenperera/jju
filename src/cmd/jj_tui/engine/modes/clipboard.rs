use super::{Effect, ModeState, ReduceCtx};
use crate::cmd::jj_tui::state::MessageKind;

pub(super) fn copy_branch_selection(ctx: &mut ReduceCtx<'_>, key: char) {
    let branch = match ctx.mode {
        ModeState::ClipboardBranchSelect(state) => state.branch_for_key(key).map(str::to_owned),
        _ => None,
    };

    let Some(branch) = branch else {
        ctx.set_status("Unknown branch selection", MessageKind::Warning);
        return;
    };

    ctx.effects.push(Effect::CopyToClipboard {
        value: branch.clone(),
        success: format!("Copied branch {branch} to clipboard"),
    });
    *ctx.mode = ModeState::Normal;
}
