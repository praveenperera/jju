//! Effects runner for jj_tui
//!
//! The runner executes effects by performing IO operations.
//! It handles terminal restore/init for operations that need the terminal.

mod bookmarks;
mod context;
mod dispatch;
mod error;
mod git;
mod interactive;
mod operations;
mod revision;

use super::effect::Effect;
pub use context::{RunCtx, RunResult};
use ratatui::DefaultTerminal;

/// Execute a list of effects
pub fn run_effects(
    mut ctx: RunCtx<'_>,
    effects: Vec<Effect>,
    terminal: &mut DefaultTerminal,
) -> RunResult {
    for effect in effects {
        dispatch::run_effect(&mut ctx, effect, terminal);
    }

    ctx.result
}
