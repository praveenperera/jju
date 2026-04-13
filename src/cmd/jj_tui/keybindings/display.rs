mod keys;
mod prefix;

#[derive(Debug, Clone)]
pub struct PrefixMenuView {
    pub title: &'static str,
    pub items: Vec<(String, &'static str)>,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyFormat {
    Space,
    Concat,
    SecondKeyOnly,
}

pub use keys::{display_keys_for_command, display_keys_joined};
pub use prefix::prefix_menu;

pub(in crate::cmd::jj_tui::keybindings) use keys::{
    first_key, first_key_any_pending, join_keys, keys_for_label,
};
