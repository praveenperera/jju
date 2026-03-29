use super::{Binding, DisplayKind, KeyPattern, ModeId};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

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

pub fn prefix_menu(mode: ModeId, pending: char) -> Option<PrefixMenuView> {
    let title = super::prefix_title(pending)?;
    let mut items = Vec::new();
    for binding in super::bindings() {
        if binding.mode != mode || binding.pending_prefix != Some(pending) {
            continue;
        }
        if binding.display != DisplayKind::Primary {
            continue;
        }
        items.push((
            format_binding_key(binding, KeyFormat::SecondKeyOnly),
            binding.label,
        ));
    }
    Some(PrefixMenuView { title, items })
}

pub fn format_binding_key(binding: &Binding, fmt: KeyFormat) -> String {
    let key = display_key_pattern(&binding.key);
    match (binding.pending_prefix, fmt) {
        (Some(_prefix), KeyFormat::SecondKeyOnly) => key,
        (Some(prefix), KeyFormat::Space) => format!("{prefix} {key}"),
        (Some(prefix), KeyFormat::Concat) => format!("{prefix}{key}"),
        (None, _) => key,
    }
}

pub fn display_keys_for_label(
    mode: ModeId,
    pending_prefix: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_format: KeyFormat,
) -> Vec<String> {
    keys_for_label(mode, pending_prefix, label, include_aliases, chord_format)
}

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

pub(super) fn display_key_pattern(key: &KeyPattern) -> String {
    match key {
        KeyPattern::Exact {
            code,
            required_mods,
        } => display_key_code(*code, *required_mods),
        KeyPattern::AnyChar => "type".to_string(),
    }
}

fn display_key_code(code: KeyCode, mods: KeyModifiers) -> String {
    let base = match code {
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::Char(c) => c.to_string(),
        other => format!("{other:?}"),
    };

    if mods.contains(KeyModifiers::CONTROL) {
        format!("Ctrl+{base}")
    } else {
        base
    }
}

fn find_bindings(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
) -> impl Iterator<Item = &'static Binding> {
    super::bindings()
        .iter()
        .filter(move |b| b.mode == mode && b.pending_prefix == pending && b.label == label)
}

fn find_bindings_any_pending(mode: ModeId, label: &str) -> impl Iterator<Item = &'static Binding> {
    super::bindings()
        .iter()
        .filter(move |binding| binding.mode == mode && binding.label == label)
}

pub(super) fn first_key(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
    kind: DisplayKind,
) -> Option<String> {
    find_bindings(mode, pending, label)
        .find(|b| b.display == kind)
        .map(|b| format_binding_key(b, KeyFormat::Space))
}

pub(super) fn first_key_any_pending(
    mode: ModeId,
    label: &str,
    kind: DisplayKind,
    format: KeyFormat,
) -> Option<String> {
    find_bindings_any_pending(mode, label)
        .find(|binding| binding.display == kind)
        .map(|binding| format_binding_key(binding, format))
}

pub(super) fn keys_for_label(
    mode: ModeId,
    pending: Option<char>,
    label: &str,
    include_aliases: bool,
    chord_fmt: KeyFormat,
) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for binding in find_bindings(mode, pending, label) {
        if !include_aliases && binding.display != DisplayKind::Primary {
            continue;
        }
        let key = match (binding.pending_prefix, chord_fmt) {
            (Some(_), KeyFormat::SecondKeyOnly) => {
                format_binding_key(binding, KeyFormat::SecondKeyOnly)
            }
            (Some(_), KeyFormat::Concat) => format_binding_key(binding, KeyFormat::Concat),
            (Some(_), KeyFormat::Space) => format_binding_key(binding, KeyFormat::Space),
            (None, _) => display_key_pattern(&binding.key),
        };
        keys.push(key);
    }
    keys
}

pub(super) fn join_keys(keys: &[String], sep: &str) -> String {
    keys.join(sep)
}
