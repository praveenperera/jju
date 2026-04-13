use super::super::super::{Binding, CommandSpec, KeyPattern, ModeId};
use super::super::{describe_binding_key, mode_name};
use ahash::{HashMap, HashMapExt};
use eyre::{Result, eyre};

pub(super) fn register_prefix_titles(
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

pub(super) fn validate_unique_binding(
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

pub(super) fn validate_pending_prefixes(bindings: &[Binding]) -> Result<()> {
    let mut available = HashMap::<ModeId, Vec<char>>::new();
    for binding in bindings {
        if binding.pending_prefix.is_none()
            && let super::super::super::ActionTemplate::Fixed(
                super::super::super::super::action::Action::SetPendingKey(prefix),
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
