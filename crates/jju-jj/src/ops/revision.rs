use super::command::run_with_stderr;
use duct::cmd;
use eyre::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct RevisionOps;

impl RevisionOps {
    pub fn edit(self, rev: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "edit", rev))
    }

    pub fn new_commit(self, rev: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "new", rev))
    }

    pub fn commit(self, message: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "commit", "-m", message))
    }

    pub fn abandon(self, revset: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "abandon", revset))
    }
}
