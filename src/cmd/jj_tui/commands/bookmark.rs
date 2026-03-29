use super::common::run_with_stderr;
use duct::cmd;
use eyre::Result;

pub fn set(name: &str, rev: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "bookmark", "set", name, "-r", rev))
}

pub fn set_allow_backwards(name: &str, rev: &str) -> Result<()> {
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

pub fn delete(name: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "bookmark", "delete", name))
}

pub fn track(name: &str) -> Result<()> {
    let remote_ref = format!("{name}@origin");
    run_with_stderr(cmd!("jj", "bookmark", "track", &remote_ref))
}
