use super::catalog;
use super::config::{self, BindingOverride};
use super::{Binding, BindingSpec, KeyPattern, ModeId};
use ahash::{HashMap, HashMapExt};
use eyre::{Result, eyre};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug)]
struct Registry {
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

pub fn prefix_title(prefix: char) -> Option<&'static str> {
    registry_load().registry.prefix_titles.get(&prefix).copied()
}

pub fn is_known_prefix(prefix: char) -> bool {
    prefix_title(prefix).is_some()
}

fn registry_load() -> &'static RegistryLoad {
    REGISTRY.get_or_init(load_registry)
}

fn load_registry() -> RegistryLoad {
    load_registry_with_warning(config_path().as_deref())
}

fn load_registry_with_warning(path: Option<&Path>) -> RegistryLoad {
    match build_registry(path) {
        Ok(registry) => RegistryLoad {
            registry,
            warning: None,
        },
        Err(error) => RegistryLoad {
            registry: builtin_registry(),
            warning: Some(error.to_string()),
        },
    }
}

fn build_registry(path: Option<&Path>) -> Result<Registry> {
    let mut specs = catalog::command_specs();
    if let Some(path) = path
        && path.exists()
    {
        apply_overrides(&mut specs, config::load_overrides(path)?)?;
    }
    compile_registry(specs)
}

fn builtin_registry() -> Registry {
    match compile_registry(catalog::command_specs()) {
        Ok(registry) => registry,
        Err(error) => panic!("invalid built-in keybindings: {error}"),
    }
}

fn apply_overrides(specs: &mut [BindingSpec], overrides: Vec<BindingOverride>) -> Result<()> {
    for binding in overrides {
        let Some(spec) = specs
            .iter_mut()
            .find(|spec| spec.mode == binding.mode && spec.label == binding.command)
        else {
            return Err(eyre!(
                "unknown keybinding command `{}` for mode `{}`",
                binding.command,
                mode_name(binding.mode)
            ));
        };
        spec.keys = binding.keys;
    }
    Ok(())
}

fn compile_registry(specs: Vec<BindingSpec>) -> Result<Registry> {
    let mut bindings = Vec::new();
    let mut prefix_titles = HashMap::new();
    let mut seen = HashMap::<(ModeId, Option<char>, KeyPattern), &'static str>::new();

    for spec in &specs {
        if let Some(title) = spec.effective_prefix_title() {
            for key in &spec.keys {
                if let Some(prefix) = key.prefix() {
                    match prefix_titles.get(&prefix) {
                        Some(existing) if *existing != title => {
                            return Err(eyre!(
                                "prefix `{prefix}` maps to both `{existing}` and `{title}`"
                            ));
                        }
                        Some(_) => {}
                        None => {
                            prefix_titles.insert(prefix, title);
                        }
                    }
                }
            }
        }

        for binding in spec.compile_bindings()? {
            let key = (binding.mode, binding.pending_prefix, binding.key);
            if let Some(existing) = seen.insert(key, binding.label) {
                return Err(eyre!(
                    "duplicate keybinding for mode `{}` on `{}` between `{}` and `{}`",
                    mode_name(binding.mode),
                    describe_binding_key(binding.pending_prefix, binding.key),
                    existing,
                    binding.label
                ));
            }
            bindings.push(binding);
        }
    }

    validate_pending_prefixes(&bindings)?;

    Ok(Registry {
        bindings,
        prefix_titles,
    })
}

fn config_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("jju/keybindings.toml"));
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/jju/keybindings.toml"))
}

fn describe_binding_key(pending: Option<char>, key: KeyPattern) -> String {
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

fn mode_name(mode: ModeId) -> &'static str {
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

fn validate_pending_prefixes(bindings: &[Binding]) -> Result<()> {
    let mut available = HashMap::<ModeId, Vec<char>>::new();
    for binding in bindings {
        if binding.pending_prefix.is_none()
            && let super::ActionTemplate::Fixed(super::super::action::Action::SetPendingKey(prefix)) =
                &binding.action
        {
            available.entry(binding.mode).or_default().push(*prefix);
        }
    }

    for binding in bindings {
        if let Some(prefix) = binding.pending_prefix {
            let has_prefix = available
                .get(&binding.mode)
                .is_some_and(|prefixes| prefixes.contains(&prefix));
            if !has_prefix {
                return Err(eyre!(
                    "binding `{}` in mode `{}` uses chord prefix `{prefix}` without a matching prefix key",
                    binding.label,
                    mode_name(binding.mode)
                ));
            }
        }
    }

    Ok(())
}

pub(crate) fn warning_duration() -> Duration {
    Duration::from_secs(10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_CONFIG_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn test_override_replaces_builtin_keys_for_command() {
        let path = write_temp_config(
            r#"
version = 2

[[binding]]
mode = "normal"
command = "down"
keys = [["["]]
"#,
        );

        let registry = build_registry(Some(&path)).expect("load registry");
        assert!(
            registry
                .bindings
                .iter()
                .any(|binding| binding.mode == ModeId::Normal
                    && binding.label == "down"
                    && matches!(
                        binding.key,
                        KeyPattern::Exact {
                            code: ratatui::crossterm::event::KeyCode::Char('['),
                            ..
                        }
                    ))
        );
        assert!(
            !registry
                .bindings
                .iter()
                .any(|binding| binding.mode == ModeId::Normal
                    && binding.label == "down"
                    && matches!(
                        binding.key,
                        KeyPattern::Exact {
                            code: ratatui::crossterm::event::KeyCode::Char('j'),
                            ..
                        }
                    ))
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_override_rejects_duplicate_keys() {
        let path = write_temp_config(
            r#"
version = 2

[[binding]]
mode = "normal"
command = "down"
keys = [["k"]]
"#,
        );

        let error = build_registry(Some(&path)).expect_err("expected duplicate key error");
        assert!(error.to_string().contains("duplicate keybinding"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_invalid_config_falls_back_with_warning() {
        let path = write_temp_config(
            r#"
version = 2

[[binding]]
mode = "normal"
command = "missing"
keys = [["x"]]
"#,
        );

        let load = load_registry_with_warning(Some(&path));
        assert!(load.warning.is_some());
        assert!(
            load.registry
                .bindings
                .iter()
                .any(|binding| binding.mode == ModeId::Normal && binding.label == "down")
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_prefix_override_requires_matching_prefix_binding() {
        let path = write_temp_config(
            r#"
version = 2

[[binding]]
mode = "normal"
command = "fetch"
keys = [["x", "f"]]
"#,
        );

        let error = build_registry(Some(&path)).expect_err("expected invalid prefix override");
        assert!(error.to_string().contains("matching prefix key"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_prefix_override_updates_prefix_title() {
        let path = write_temp_config(
            r#"
version = 2

[[binding]]
mode = "normal"
command = "git"
keys = [["X"]]

[[binding]]
mode = "normal"
command = "fetch"
keys = [["X", "f"]]

[[binding]]
mode = "normal"
command = "import"
keys = [["X", "i"]]

[[binding]]
mode = "normal"
command = "export"
keys = [["X", "e"]]

[[binding]]
mode = "normal"
command = "resolve_divergence"
keys = [["X", "r"]]

[[binding]]
mode = "normal"
command = "create_pr"
keys = [["X", "p"]]
"#,
        );

        let registry = build_registry(Some(&path)).expect("load registry");
        assert_eq!(registry.prefix_titles.get(&'X').copied(), Some("git"));

        let _ = fs::remove_file(path);
    }

    fn write_temp_config(contents: &str) -> PathBuf {
        let suffix = TEMP_CONFIG_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("jju-keybindings-{pid}-{suffix}.toml"));
        fs::write(&path, contents).expect("write config");
        path
    }
}
