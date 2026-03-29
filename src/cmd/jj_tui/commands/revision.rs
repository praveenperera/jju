use super::common::run_with_stderr;
use duct::cmd;
use eyre::Result;

pub fn edit(rev: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "edit", rev))
}

pub fn new(rev: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "new", rev))
}

pub fn commit(message: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "commit", "-m", message))
}

pub fn abandon(revset: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "abandon", revset))
}
