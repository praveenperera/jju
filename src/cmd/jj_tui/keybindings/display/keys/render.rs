use super::super::KeyFormat;
use super::lookup::{keys_for_command, keys_for_label};
use crate::cmd::jj_tui::keybindings::ModeId;

pub fn display_keys_joined(
    mode: ModeId,
    pending_prefix: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_format: KeyFormat,
    sep: &str,
) -> String {
    join_keys(
        &display_keys_for_label(mode, pending_prefix, label, include_aliases, chord_format),
        sep,
    )
}

pub fn display_keys_for_command(
    mode: ModeId,
    label: &str,
    include_aliases: bool,
    chord_format: KeyFormat,
) -> Vec<String> {
    keys_for_command(mode, label, include_aliases, chord_format)
}

pub(super) fn display_keys_for_label(
    mode: ModeId,
    pending_prefix: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_format: KeyFormat,
) -> Vec<String> {
    keys_for_label(mode, pending_prefix, label, include_aliases, chord_format)
}

pub(in crate::cmd::jj_tui::keybindings) fn join_keys(keys: &[String], sep: &str) -> String {
    keys.join(sep)
}
