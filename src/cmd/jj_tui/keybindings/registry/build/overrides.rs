use super::super::super::CommandSpec;
use super::super::super::config::BindingOverride;
use super::super::mode_name;
use eyre::{Result, eyre};

pub(super) fn apply_overrides(
    specs: &mut [CommandSpec],
    overrides: Vec<BindingOverride>,
) -> Result<()> {
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
