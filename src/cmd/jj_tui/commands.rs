//! JJ command execution
//!
//! This module centralizes JJ command execution patterns.
//! It provides a cleaner API than using cmd!() directly.

use duct::cmd;
use eyre::Result;

/// Run a command and include stderr in the error message on failure
fn run_with_stderr(expr: duct::Expression) -> Result<()> {
    let output = expr.stdout_null().stderr_capture().unchecked().run()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if stderr.is_empty() {
            eyre::bail!("command failed with exit code {:?}", output.status.code());
        } else {
            eyre::bail!("{}", stderr);
        }
    }
    Ok(())
}

/// Get the current operation ID for potential undo
pub fn get_current_op_id() -> Result<String> {
    let output = cmd!("jj", "op", "log", "--limit", "1", "-T", "id", "--no-graph")
        .stdout_capture()
        .stderr_null()
        .read()?;
    Ok(output.trim().to_string())
}

/// Restore to a previous operation (undo)
pub fn restore_op(op_id: &str) -> Result<()> {
    run_with_stderr(cmd!("jj", "op", "restore", op_id))
}

/// Git operations
pub mod git {
    use super::run_with_stderr;
    use duct::cmd;
    use eyre::Result;

    pub fn push_bookmark(name: &str) -> Result<()> {
        // track the remote bookmark if it exists, ignore errors for new bookmarks
        let _ = super::bookmark::track(name);
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
}

/// Bookmark operations
pub mod bookmark {
    use super::run_with_stderr;
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
}

/// Revision operations
pub mod revision {
    use super::run_with_stderr;
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
}

/// Rebase operations
pub mod rebase {
    use super::run_with_stderr;
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
}

/// Diff operations
pub mod diff {
    use duct::cmd;
    use eyre::Result;

    pub fn get_diff(rev: &str) -> Result<String> {
        let output = cmd!("jj", "diff", "--git", "-r", rev)
            .stdout_capture()
            .stderr_null()
            .read()?;
        Ok(output)
    }

    pub fn get_stats(change_id: &str) -> Result<String> {
        let output = cmd!("jj", "diff", "--stat", "-r", change_id)
            .stdout_capture()
            .stderr_null()
            .read()?;
        Ok(output)
    }
}

/// Check if working copy has conflicts
pub fn has_conflicts() -> Result<bool> {
    let output = cmd!("jj", "log", "-r", "@", "-T", r#"if(conflict, "conflict")"#)
        .stdout_capture()
        .stderr_null()
        .read()?;
    Ok(output.contains("conflict"))
}

/// Check if rev1 is an ancestor of rev2
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


/// List files with conflicts in the working copy
pub fn list_conflict_files() -> Result<Vec<String>> {
    let template = r#"conflict_files.map(|x| x ++ "\n").join("")"#;
    let output = cmd!("jj", "log", "-r", "@", "-T", template, "--no-graph")
        .stdout_capture()
        .stderr_null()
        .read()?;
    let files: Vec<String> = output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(files)
}

/// Stack sync operations
pub mod stack_sync {
    use super::run_with_stderr;
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
}

/// Resolve operations
pub mod resolve {
    use eyre::Result;
    use std::process::Command;

    /// Run jj resolve on a specific file (interactive, requires terminal)
    pub fn resolve_file(file: &str) -> Result<()> {
        Command::new("jj").args(["resolve", file]).status()?;
        Ok(())
    }
}
