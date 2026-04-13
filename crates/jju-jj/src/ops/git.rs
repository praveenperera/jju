use super::bookmark::BookmarkOps;
use super::command::run_with_stderr;
use duct::cmd;
use eyre::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct GitOps;

impl GitOps {
    pub fn fetch(self) -> Result<()> {
        run_with_stderr(cmd!("jj", "git", "fetch"))
    }

    pub fn import(self) -> Result<()> {
        run_with_stderr(cmd!("jj", "git", "import"))
    }

    pub fn export(self) -> Result<()> {
        run_with_stderr(cmd!("jj", "git", "export"))
    }

    pub fn push_all(self) -> Result<()> {
        run_with_stderr(cmd!("jj", "git", "push", "--all"))
    }

    pub fn push_bookmark(self, bookmark: &str) -> Result<()> {
        let _ = BookmarkOps.track(bookmark);
        run_with_stderr(cmd!("jj", "git", "push", "--bookmark", bookmark))
    }

    pub fn has_open_pr(self, bookmark: &str) -> bool {
        cmd!("gh", "pr", "view", bookmark, "--json", "url")
            .stdout_null()
            .stderr_null()
            .unchecked()
            .run()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn push_and_pr(self, bookmark: &str) -> Result<bool> {
        self.push_bookmark(bookmark)?;
        if self.has_open_pr(bookmark) {
            run_with_stderr(cmd!("gh", "pr", "view", bookmark, "--web"))?;
            Ok(true)
        } else {
            run_with_stderr(cmd!("gh", "pr", "create", "--head", bookmark, "--web"))?;
            Ok(false)
        }
    }
}
