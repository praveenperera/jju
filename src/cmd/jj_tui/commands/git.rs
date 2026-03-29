use super::bookmark;
use super::common::run_with_stderr;
use duct::cmd;
use eyre::Result;

pub fn push_bookmark(name: &str) -> Result<()> {
    let _ = bookmark::track(name);
    run_with_stderr(cmd!("jj", "git", "push", "--bookmark", name))
}

pub fn import() -> Result<()> {
    run_with_stderr(cmd!("jj", "git", "import"))
}

pub fn export() -> Result<()> {
    run_with_stderr(cmd!("jj", "git", "export"))
}

pub fn push_all() -> Result<()> {
    run_with_stderr(cmd!("jj", "git", "push", "--all"))
}

pub fn fetch() -> Result<()> {
    run_with_stderr(cmd!("jj", "git", "fetch"))
}

pub fn has_open_pr(bookmark: &str) -> bool {
    cmd!("gh", "pr", "view", bookmark, "--json", "url")
        .stdout_null()
        .stderr_null()
        .unchecked()
        .run()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Push a bookmark and create or open its PR
/// Returns true if an existing PR was opened, false if a new one was created
pub fn push_and_pr(bookmark: &str) -> Result<bool> {
    push_bookmark(bookmark)?;
    if has_open_pr(bookmark) {
        run_with_stderr(cmd!("gh", "pr", "view", bookmark, "--web"))?;
        Ok(true)
    } else {
        run_with_stderr(cmd!("gh", "pr", "create", "--head", bookmark, "--web"))?;
        Ok(false)
    }
}
