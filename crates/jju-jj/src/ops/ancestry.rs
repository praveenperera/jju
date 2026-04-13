use duct::cmd;
use eyre::Result;

pub fn is_ancestor(rev1: &str, rev2: &str) -> Result<bool> {
    let revset = format!("{rev1} & ::({rev2})");
    let output = cmd!(
        "jj",
        "log",
        "-r",
        revset,
        "--no-graph",
        "-T",
        "change_id",
        "--limit",
        "1"
    )
    .stdout_capture()
    .stderr_null()
    .read()?;
    Ok(!output.trim().is_empty())
}
