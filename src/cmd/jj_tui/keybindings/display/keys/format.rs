use super::super::KeyFormat;
use crate::cmd::jj_tui::keybindings::{Binding, KeyPattern};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

pub(super) fn format_binding_key(binding: &Binding, fmt: KeyFormat) -> String {
    let key = display_key_pattern(&binding.key);
    match (binding.pending_prefix, fmt) {
        (Some(_prefix), KeyFormat::SecondKeyOnly) => key,
        (Some(prefix), KeyFormat::Space) => format!("{prefix} {key}"),
        (Some(prefix), KeyFormat::Concat) => format!("{prefix}{key}"),
        (None, _) => key,
    }
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
