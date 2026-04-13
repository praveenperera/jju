mod parse;

use super::{KeySequence, ModeId};
use eyre::{Result, eyre};
use std::path::Path;

#[derive(Debug, Clone)]
pub(super) struct BindingOverride {
    pub mode: ModeId,
    pub command: String,
    pub keys: Vec<KeySequence>,
}

pub(super) fn load_overrides(path: &Path) -> Result<Vec<BindingOverride>> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| eyre!("failed to read {}: {error}", path.display()))?;
    parse_overrides(&text)
}

pub(super) fn parse_overrides(text: &str) -> Result<Vec<BindingOverride>> {
    parse::parse_overrides(text)
}

#[cfg(test)]
mod tests;
