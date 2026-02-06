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

    pub fn single(source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-r", source, "-A", dest))
    }

    pub fn single_with_next(source: &str, dest: &str, next: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-r", source, "-A", dest, "-B", next))
    }

    pub fn with_descendants(source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-s", source, "-A", dest))
    }

    pub fn with_descendants_and_next(source: &str, dest: &str, next: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-s", source, "-A", dest, "-B", next))
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

/// Get the first child of a revision (if any)
pub fn get_first_child(rev: &str) -> Result<Option<String>> {
    let revset = format!("children({rev})");
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
    let trimmed = output.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
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
