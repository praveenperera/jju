use super::BindingOverride;
use crate::cmd::jj_tui::keybindings::{KeyDef, KeySequence, ModeId};
use eyre::{Result, bail, eyre};
use ratatui::crossterm::event::KeyCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeybindingsFile {
    version: u32,
    #[serde(default)]
    binding: Vec<BindingOverrideToml>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingOverrideToml {
    mode: String,
    command: String,
    keys: Vec<Vec<String>>,
}

pub(super) fn parse_overrides(text: &str) -> Result<Vec<BindingOverride>> {
    let file: KeybindingsFile =
        toml::from_str(text).map_err(|error| eyre!("failed to parse keybindings TOML: {error}"))?;

    if file.version != 2 {
        bail!(
            "unsupported keybindings config version {}, expected 2",
            file.version
        );
    }

    file.binding
        .into_iter()
        .map(parse_binding_override)
        .collect()
}

fn parse_binding_override(binding: BindingOverrideToml) -> Result<BindingOverride> {
    if binding.keys.is_empty() {
        bail!(
            "binding override `{}` for mode `{}` must define at least one key sequence",
            binding.command,
            binding.mode
        );
    }

    Ok(BindingOverride {
        mode: parse_mode(&binding.mode)?,
        command: binding.command,
        keys: binding
            .keys
            .into_iter()
            .map(parse_sequence)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn parse_mode(mode: &str) -> Result<ModeId> {
    match mode {
        "normal" => Ok(ModeId::Normal),
        "help" => Ok(ModeId::Help),
        "diff" => Ok(ModeId::Diff),
        "confirm" => Ok(ModeId::Confirm),
        "selecting" => Ok(ModeId::Selecting),
        "rebase" => Ok(ModeId::Rebase),
        "squash" => Ok(ModeId::Squash),
        "moving_bookmark" => Ok(ModeId::MovingBookmark),
        "bookmark_select" => Ok(ModeId::BookmarkSelect),
        "bookmark_picker" => Ok(ModeId::BookmarkPicker),
        "push_select" => Ok(ModeId::PushSelect),
        "conflicts" => Ok(ModeId::Conflicts),
        _ => bail!("unknown keybinding mode `{mode}`"),
    }
}

fn parse_sequence(tokens: Vec<String>) -> Result<KeySequence> {
    match tokens.len() {
        1 => Ok(KeySequence::Single(parse_key(&tokens[0])?)),
        2 => {
            let prefix = parse_key(&tokens[0])?.plain_char().ok_or_else(|| {
                eyre!(
                    "chord prefixes must be plain character keys, got `{}`",
                    tokens[0]
                )
            })?;
            let suffix = parse_key(&tokens[1])?;
            if matches!(suffix, KeyDef::AnyChar) {
                bail!("`AnyChar` is not supported as the second step of a chord");
            }
            Ok(KeySequence::Chord(prefix, suffix))
        }
        _ => bail!("key sequences must have one or two steps"),
    }
}

fn parse_key(token: &str) -> Result<KeyDef> {
    match token {
        "Enter" => Ok(KeyDef::Key(KeyCode::Enter)),
        "Esc" => Ok(KeyDef::Key(KeyCode::Esc)),
        "Tab" => Ok(KeyDef::Key(KeyCode::Tab)),
        "Backspace" => Ok(KeyDef::Key(KeyCode::Backspace)),
        "Delete" | "Del" => Ok(KeyDef::Key(KeyCode::Delete)),
        "Up" => Ok(KeyDef::Key(KeyCode::Up)),
        "Down" => Ok(KeyDef::Key(KeyCode::Down)),
        "Left" => Ok(KeyDef::Key(KeyCode::Left)),
        "Right" => Ok(KeyDef::Key(KeyCode::Right)),
        "Space" => Ok(KeyDef::Char(' ')),
        "AnyChar" => Ok(KeyDef::AnyChar),
        _ if token.starts_with("Ctrl+") => {
            let suffix = &token["Ctrl+".len()..];
            let mut chars = suffix.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Ok(KeyDef::Ctrl(ch.to_ascii_lowercase())),
                _ => bail!("control bindings must target exactly one character, got `{token}`"),
            }
        }
        _ => {
            let mut chars = token.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Ok(KeyDef::Char(ch)),
                _ => bail!("unsupported key token `{token}`"),
            }
        }
    }
}
