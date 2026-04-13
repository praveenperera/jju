//! Engine for jj_tui
//!
//! The engine is a pure function that processes actions and produces effects.
//! It mutates state but performs no IO.

mod bookmarks;
mod commands;
mod lifecycle;
mod modes;
mod navigation;
mod rebase;
mod selection;
#[cfg(test)]
mod tests;

use super::action::{Action, ActionDomain};
use super::effect::Effect;
use super::state::{MessageKind, ModeState};
use super::tree::TreeState;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct ReduceCtx<'a> {
    pub tree: &'a mut TreeState,
    pub mode: &'a mut ModeState,
    pub should_quit: &'a mut bool,
    pub split_view: &'a mut bool,
    pub pending_key: &'a mut Option<char>,
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
        resources: ReduceResources<'a>,
    ) -> Self {
        Self {
            tree,
            mode,
            should_quit,
            split_view,
            pending_key,
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
    let clear_pending_key = action.clears_pending_key();

    dispatch_by_domain(&mut ctx, action);

    if clear_pending_key {
        *ctx.pending_key = None;
    }

    ctx.effects
}
fn dispatch_by_domain(ctx: &mut ReduceCtx<'_>, action: Action) {
    match action.domain() {
        ActionDomain::Navigation => navigation::handle(ctx, action),
        ActionDomain::Modes => modes::handle(ctx, action),
        ActionDomain::Bookmarks => bookmarks::handle(ctx, action),
        ActionDomain::Commands => commands::handle(ctx, action),
        ActionDomain::Lifecycle => lifecycle::handle(ctx, action),
    }
}
