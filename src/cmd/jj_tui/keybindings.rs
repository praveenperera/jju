//! Centralized keymap and keybinding rendering
//!
//! This module is the single source of truth for:
//! - Mapping (ModeState, KeyEvent) -> Action
//! - Status bar / context help hints
//! - Prefix (chord) menus
//! - Help popup content

mod bindings;
mod catalog;
mod config;
mod dispatch;
mod display;
mod help;
mod hints;
mod registry;
mod spec;
mod types;

pub(crate) use dispatch::dispatch_key;
pub(crate) use display::{KeyFormat, display_keys_joined, prefix_menu};
pub(crate) use help::build_help_view;
pub(crate) use hints::{StatusHintContext, status_bar_hints};
pub(crate) use registry::{bindings, commands, initialize, warning_duration};
pub use registry::{is_known_prefix, prefix_title};
pub(crate) use spec::{BindingBehavior, BindingSpec, CommandSpec, KeySequence};
pub(crate) use types::{ActionTemplate, Binding, DisplayKind, KeyDef, KeyPattern};
pub use types::{ModeId, mode_id_from_state};

#[cfg(test)]
mod tests;
