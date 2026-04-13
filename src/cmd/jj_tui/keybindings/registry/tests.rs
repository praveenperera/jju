use super::build::{build_registry, load_registry_with_warning};
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
