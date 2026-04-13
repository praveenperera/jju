use super::super::super::CommandSpec;
use super::super::Registry;
use super::validate;
use ahash::{HashMap, HashMapExt};
use eyre::Result;

pub(super) fn compile_registry(commands: Vec<CommandSpec>) -> Result<Registry> {
    let mut bindings = Vec::new();
    let mut prefix_titles = HashMap::new();
    let mut seen = HashMap::default();

    for command in &commands {
        validate::register_prefix_titles(&mut prefix_titles, command)?;
        for binding in command.compile_bindings()? {
            validate::validate_unique_binding(&mut seen, &binding)?;
            bindings.push(binding);
        }
    }

    validate::validate_pending_prefixes(&bindings)?;

    Ok(Registry {
        commands,
        bindings,
        prefix_titles,
    })
}
