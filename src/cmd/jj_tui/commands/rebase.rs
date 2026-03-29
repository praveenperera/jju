use super::common::run_with_stderr;
use duct::cmd;
use eyre::Result;

/// Inline rebase: insert source after dest, reparenting dest's children under source
pub fn single(source: &str, dest: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "rebase", "-r", source, "-A", dest))
}

/// Inline rebase with descendants
pub fn with_descendants(source: &str, dest: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "rebase", "-s", source, "-A", dest))
}

/// Fork rebase: set dest as parent, dest's children unaffected
pub fn single_fork(source: &str, dest: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "rebase", "-r", source, "-d", dest))
}

/// Fork rebase with descendants
pub fn with_descendants_fork(source: &str, dest: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "rebase", "-s", source, "-d", dest))
}

/// Rebase single commit onto trunk()
pub fn single_onto_trunk(source: &str) -> Result<()> {
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

/// Rebase commit with descendants onto trunk()
pub fn with_descendants_onto_trunk(source: &str) -> Result<()> {
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
