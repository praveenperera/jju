mod application;
mod command;
mod diff;
mod plan;
mod preview;
mod repo;
mod selection;
mod workflow;

pub use command::SplitHunkCommand;
use eyre::{Result, eyre};

#[derive(Debug, Clone)]
pub struct SplitHunkOptions {
    pub message: Option<String>,
    pub revision: String,
    pub file_filter: Option<String>,
    pub lines: Option<String>,
    pub hunks: Option<String>,
    pub pattern: Option<String>,
    pub preview: bool,
    pub invert: bool,
    pub dry_run: bool,
}

impl SplitHunkOptions {
    fn commit_message(&self) -> Result<&str> {
        if self.preview {
            return Ok("");
        }

        self.message
            .as_deref()
            .ok_or_else(|| eyre!("--message is required unless using --preview"))
    }
}
