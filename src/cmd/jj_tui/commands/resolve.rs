use eyre::Result;
use std::process::Command;

/// Run jj resolve on a specific file (interactive, requires terminal)
pub fn resolve_file(file: &str) -> Result<()> {
    Command::new("jj").args(["resolve", file]).status()?;
    Ok(())
}
