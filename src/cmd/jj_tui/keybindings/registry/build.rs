use super::super::catalog;
use super::super::config::{self, BindingOverride};
use super::super::{Binding, CommandSpec, KeyPattern, ModeId};
use super::{Registry, RegistryLoad, describe_binding_key, mode_name};
use ahash::{HashMap, HashMapExt};
use eyre::{Result, eyre};
use std::path::Path;

pub(super) fn load_registry_with_warning(path: Option<&Path>) -> RegistryLoad {
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

pub(super) fn build_registry(path: Option<&Path>) -> Result<Registry> {
    let mut specs = catalog::command_specs();
    if let Some(path) = path
        && path.exists()
    {
        apply_overrides(&mut specs, config::load_overrides(path)?)?;
    }
    compile_registry(specs)
}

pub(super) fn builtin_registry() -> Registry {
    match compile_registry(catalog::command_specs()) {
        Ok(registry) => registry,
        Err(error) => panic!("invalid built-in keybindings: {error}"),
    }
}

fn apply_overrides(specs: &mut [CommandSpec], overrides: Vec<BindingOverride>) -> Result<()> {
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

fn compile_registry(commands: Vec<CommandSpec>) -> Result<Registry> {
    let mut bindings = Vec::new();
    let mut prefix_titles = HashMap::new();
    let mut seen = HashMap::<(ModeId, Option<char>, KeyPattern), &'static str>::new();

    for command in &commands {
        register_prefix_titles(&mut prefix_titles, command)?;
        for binding in command.compile_bindings()? {
            validate_unique_binding(&mut seen, &binding)?;
            bindings.push(binding);
        }
    }

    validate_pending_prefixes(&bindings)?;

    Ok(Registry {
        commands,
        bindings,
        prefix_titles,
    })
}

fn register_prefix_titles(
    prefix_titles: &mut HashMap<char, &'static str>,
    command: &CommandSpec,
) -> Result<()> {
    let Some(title) = command.effective_prefix_title() else {
        return Ok(());
    };

    for key in &command.keys {
        let Some(prefix) = key.prefix() else {
            continue;
        };
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

    Ok(())
}

fn validate_unique_binding(
    seen: &mut HashMap<(ModeId, Option<char>, KeyPattern), &'static str>,
    binding: &Binding,
) -> Result<()> {
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
    Ok(())
}

fn validate_pending_prefixes(bindings: &[Binding]) -> Result<()> {
    let mut available = HashMap::<ModeId, Vec<char>>::new();
    for binding in bindings {
        if binding.pending_prefix.is_none()
            && let super::super::ActionTemplate::Fixed(
                super::super::super::action::Action::SetPendingKey(prefix),
            ) = &binding.action
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
