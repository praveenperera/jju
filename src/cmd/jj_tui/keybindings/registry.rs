mod build;
#[cfg(test)]
mod tests;

use super::{Binding, CommandSpec, KeyPattern, ModeId};
use ahash::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug)]
struct Registry {
    commands: Vec<CommandSpec>,
    bindings: Vec<Binding>,
    prefix_titles: HashMap<char, &'static str>,
}

#[derive(Debug)]
struct RegistryLoad {
    registry: Registry,
    warning: Option<String>,
}

static REGISTRY: OnceLock<RegistryLoad> = OnceLock::new();

pub(crate) fn initialize() -> Option<String> {
    registry_load().warning.clone()
}

pub(crate) fn bindings() -> &'static [Binding] {
    &registry_load().registry.bindings
}

pub(crate) fn commands() -> &'static [CommandSpec] {
    &registry_load().registry.commands
}

pub fn prefix_title(prefix: char) -> Option<&'static str> {
    registry_load().registry.prefix_titles.get(&prefix).copied()
}

pub fn is_known_prefix(prefix: char) -> bool {
    prefix_title(prefix).is_some()
}

pub(super) fn describe_binding_key(pending: Option<char>, key: KeyPattern) -> String {
    match (pending, key) {
        (
            Some(prefix),
            KeyPattern::Exact {
                code,
                required_mods,
            },
        ) => {
            format!("{prefix} {}", describe_key(code, required_mods))
        }
        (
            None,
            KeyPattern::Exact {
                code,
                required_mods,
            },
        ) => describe_key(code, required_mods),
        (_, KeyPattern::AnyChar) => "AnyChar".to_string(),
    }
}

pub(super) fn mode_name(mode: ModeId) -> &'static str {
    match mode {
        ModeId::Normal => "normal",
        ModeId::Help => "help",
        ModeId::Diff => "diff",
        ModeId::Confirm => "confirm",
        ModeId::Selecting => "selecting",
        ModeId::Rebase => "rebase",
        ModeId::Squash => "squash",
        ModeId::MovingBookmark => "moving_bookmark",
        ModeId::BookmarkSelect => "bookmark_select",
        ModeId::BookmarkPicker => "bookmark_picker",
        ModeId::PushSelect => "push_select",
        ModeId::Conflicts => "conflicts",
    }
}

pub(crate) fn warning_duration() -> Duration {
    Duration::from_secs(10)
}

fn registry_load() -> &'static RegistryLoad {
    REGISTRY.get_or_init(load_registry)
}

fn load_registry() -> RegistryLoad {
    load_registry_with_warning(config_path().as_deref())
}

fn config_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("jju/keybindings.toml"));
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/jju/keybindings.toml"))
}

fn load_registry_with_warning(path: Option<&std::path::Path>) -> RegistryLoad {
    build::load_registry_with_warning(path)
}

fn describe_key(
    code: ratatui::crossterm::event::KeyCode,
    mods: ratatui::crossterm::event::KeyModifiers,
) -> String {
    if mods.contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
        && let ratatui::crossterm::event::KeyCode::Char(ch) = code
    {
        return format!("Ctrl+{ch}");
    }

    match code {
        ratatui::crossterm::event::KeyCode::Char(' ') => "Space".to_string(),
        ratatui::crossterm::event::KeyCode::Char(ch) => ch.to_string(),
        other => format!("{other:?}"),
    }
}
