mod render;
mod scenario;

use super::catalog;

pub use scenario::StatusHintContext;

pub fn status_bar_hints(ctx: &StatusHintContext) -> String {
    render::render_specs(
        ctx,
        catalog::hint_specs(scenario::scenario_for_context(ctx)),
    )
}

#[cfg(test)]
mod tests;
