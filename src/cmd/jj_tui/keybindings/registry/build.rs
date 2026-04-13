mod compile;
mod overrides;
mod validate;

use super::super::catalog;
use super::super::config;
use super::{Registry, RegistryLoad};
use eyre::Result;
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
        overrides::apply_overrides(&mut specs, config::load_overrides(path)?)?;
    }
    compile::compile_registry(specs)
}

pub(super) fn builtin_registry() -> Registry {
    match compile::compile_registry(catalog::command_specs()) {
        Ok(registry) => registry,
        Err(error) => panic!("invalid built-in keybindings: {error}"),
    }
}
