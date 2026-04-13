use super::command::run_with_stderr;
use duct::cmd;
use eyre::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct BookmarkOps;

impl BookmarkOps {
    pub fn set(self, name: &str, rev: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "bookmark", "set", name, "-r", rev))
    }

    pub fn set_allow_backwards(self, name: &str, rev: &str) -> Result<()> {
        run_with_stderr(cmd!(
            "jj",
            "bookmark",
            "set",
            name,
            "-r",
            rev,
            "--allow-backwards"
        ))
    }

    pub fn delete(self, name: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "bookmark", "delete", name))
    }

    pub fn track(self, name: &str) -> Result<()> {
        let remote_ref = format!("{name}@origin");
        run_with_stderr(cmd!("jj", "bookmark", "track", &remote_ref))
    }
}
