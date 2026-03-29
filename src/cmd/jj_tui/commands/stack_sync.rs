use super::common::run_with_stderr;
use duct::cmd;
use eyre::Result;

/// Detect the trunk branch name via jj's trunk() revset
pub fn detect_trunk_branch() -> Result<String> {
    let output = cmd!(
        "jj",
        "log",
        "-r",
        "trunk()",
        "--no-graph",
        "-T",
        r#"bookmarks.map(|x| x.name()).join("\n")"#
    )
    .stdout_capture()
    .stderr_null()
    .read()?;
    let name = output.lines().next().unwrap_or("").trim().to_string();
    if name.is_empty() {
        eyre::bail!("could not detect trunk branch");
    }
    Ok(name)
}

/// Sync trunk bookmark to match its remote tracking branch
pub fn sync_trunk_bookmark(trunk: &str) -> Result<()> {
    let remote_ref = format!("{trunk}@origin");
    run_with_stderr(cmd!("jj", "bookmark", "set", trunk, "-r", remote_ref))
}

/// Find the root commits between trunk and the working copy
pub fn find_stack_roots(trunk: &str) -> Result<Vec<String>> {
    let revset = format!("roots({trunk}..@)");
    let output = cmd!(
        "jj",
        "log",
        "-r",
        &revset,
        "--no-graph",
        "-T",
        r#"change_id.short() ++ "\n""#
    )
    .stdout_capture()
    .stderr_null()
    .read()?;
    Ok(output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

/// Get the first line of a commit's description
pub fn get_commit_description(rev: &str) -> Result<String> {
    let output = cmd!(
        "jj",
        "log",
        "-r",
        rev,
        "--no-graph",
        "-T",
        "description.first_line()"
    )
    .stdout_capture()
    .stderr_capture()
    .read()?;
    Ok(output.trim().to_string())
}

/// Rebase a stack root onto trunk with --skip-emptied
pub fn rebase_root_onto_trunk(root: &str, trunk: &str) -> Result<()> {
    run_with_stderr(cmd!(
        "jj",
        "rebase",
        "--source",
        root,
        "--onto",
        trunk,
        "--skip-emptied"
    ))
}

/// Delete tracked bookmarks that are marked as [deleted] on the remote
pub fn cleanup_deleted_bookmarks() -> Result<Vec<String>> {
    let tracked = cmd!("jj", "bookmark", "list", "--tracked")
        .stdout_capture()
        .stderr_null()
        .read()?;

    let mut deleted = Vec::new();
    for line in tracked.lines() {
        if line.contains("[deleted]")
            && let Some(bookmark) = line.split_whitespace().next()
        {
            let _ = run_with_stderr(cmd!("jj", "bookmark", "delete", bookmark));
            deleted.push(bookmark.to_string());
        }
    }
    Ok(deleted)
}
