mod modal;
mod normal;
mod operations;

use super::ActionTemplate::Fixed;
use super::{BindingBehavior, CommandSpec, KeyDef, KeySequence};
use crate::cmd::jj_tui::action::Action;

pub(super) fn builtin_commands() -> Vec<CommandSpec> {
    let mut commands = Vec::new();
    commands.extend(normal::commands());
    commands.extend(modal::commands());
    commands.extend(operations::commands());
    commands
}

pub(super) fn fixed(action: Action) -> BindingBehavior {
    BindingBehavior::Action(Fixed(action))
}

pub(super) fn single(key: KeyDef) -> KeySequence {
    KeySequence::Single(key)
}

pub(super) fn chord(prefix: char, key: KeyDef) -> KeySequence {
    KeySequence::Chord(prefix, key)
}

pub(super) fn pending_prefix(title: &'static str) -> BindingBehavior {
    BindingBehavior::PendingPrefix { title }
}
