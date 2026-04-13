use super::command::run_with_stderr;
use duct::cmd;
use eyre::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct RebaseOps;

impl RebaseOps {
    pub fn single(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-r", source, "-A", dest))
    }

    pub fn with_descendants(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-s", source, "-A", dest))
    }

    pub fn single_fork(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-r", source, "-d", dest))
    }

    pub fn with_descendants_fork(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-s", source, "-d", dest))
    }

    pub fn single_onto_trunk(self, source: &str) -> Result<()> {
        run_with_stderr(cmd!(
            "jj",
            "rebase",
            "-r",
            source,
            "-d",
            "trunk()",
            "--skip-emptied"
        ))
    }

    pub fn with_descendants_onto_trunk(self, source: &str) -> Result<()> {
        run_with_stderr(cmd!(
            "jj",
            "rebase",
            "-s",
            source,
            "-d",
            "trunk()",
            "--skip-emptied"
        ))
    }
}
