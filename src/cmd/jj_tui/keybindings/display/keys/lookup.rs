use super::super::KeyFormat;
use super::format::{display_key_pattern, format_binding_key};
use crate::cmd::jj_tui::keybindings::{Binding, DisplayKind, ModeId};

fn find_bindings(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
) -> impl Iterator<Item = &'static Binding> {
    super::super::super::bindings()
        .iter()
        .filter(move |binding| {
            binding.mode == mode && binding.pending_prefix == pending && binding.label == label
        })
}

fn find_bindings_any_pending(mode: ModeId, label: &str) -> impl Iterator<Item = &'static Binding> {
    super::super::super::bindings()
        .iter()
        .filter(move |binding| binding.mode == mode && binding.label == label)
}

pub(in crate::cmd::jj_tui::keybindings) fn first_key(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
    kind: DisplayKind,
) -> Option<String> {
    find_bindings(mode, pending, label)
        .find(|binding| binding.display == kind)
        .map(|binding| format_binding_key(binding, KeyFormat::Space))
}

pub(in crate::cmd::jj_tui::keybindings) fn first_key_any_pending(
    mode: ModeId,
    label: &str,
    kind: DisplayKind,
    format: KeyFormat,
) -> Option<String> {
    find_bindings_any_pending(mode, label)
        .find(|binding| binding.display == kind)
        .map(|binding| format_binding_key(binding, format))
}

pub(in crate::cmd::jj_tui::keybindings) fn keys_for_label(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_fmt: KeyFormat,
) -> Vec<String> {
    collect_keys(
        find_bindings(mode, pending, label),
        include_aliases,
        chord_fmt,
    )
}

pub(super) fn keys_for_command(
    mode: ModeId,
    label: &str,
    include_aliases: bool,
    chord_fmt: KeyFormat,
) -> Vec<String> {
    collect_keys(
        find_bindings_any_pending(mode, label),
        include_aliases,
        chord_fmt,
    )
}

fn collect_keys<'a>(
    bindings: impl IntoIterator<Item = &'a Binding>,
    include_aliases: bool,
    chord_fmt: KeyFormat,
) -> Vec<String> {
    bindings
        .into_iter()
        .filter(|binding| include_aliases || binding.display == DisplayKind::Primary)
        .map(|binding| match (binding.pending_prefix, chord_fmt) {
            (Some(_), KeyFormat::SecondKeyOnly) => {
                format_binding_key(binding, KeyFormat::SecondKeyOnly)
            }
            (Some(_), KeyFormat::Concat) => format_binding_key(binding, KeyFormat::Concat),
            (Some(_), KeyFormat::Space) => format_binding_key(binding, KeyFormat::Space),
            (None, _) => display_key_pattern(&binding.key),
        })
        .collect()
}
