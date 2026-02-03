//! JJ command execution
//!
//! This module centralizes JJ command execution patterns.
//! It provides a cleaner API than using cmd!() directly.

use duct::cmd;
use eyre::Result;

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
    cmd!("jj", "op", "restore", op_id)
        .stdout_null()
        .stderr_null()
        .run()?;
    Ok(())
}

/// Git operations
pub mod git {
    use duct::cmd;
    use eyre::Result;

    pub fn push_bookmark(name: &str) -> Result<()> {
        cmd!("jj", "git", "push", "--bookmark", name)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn import() -> Result<()> {
        cmd!("jj", "git", "import")
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn export() -> Result<()> {
        cmd!("jj", "git", "export")
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn push_all() -> Result<()> {
        cmd!("jj", "git", "push", "--all")
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }
}

/// Bookmark operations
pub mod bookmark {
    use duct::cmd;
    use eyre::Result;

    pub fn set(name: &str, rev: &str) -> Result<()> {
        cmd!("jj", "bookmark", "set", name, "-r", rev)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn set_allow_backwards(name: &str, rev: &str) -> Result<()> {
        cmd!("jj", "bookmark", "set", name, "-r", rev, "--allow-backwards")
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn delete(name: &str) -> Result<()> {
        cmd!("jj", "bookmark", "delete", name)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }
}

/// Revision operations
pub mod revision {
    use duct::cmd;
    use eyre::Result;

    pub fn edit(rev: &str) -> Result<()> {
        cmd!("jj", "edit", rev).stdout_null().stderr_null().run()?;
        Ok(())
    }

    pub fn new(rev: &str) -> Result<()> {
        cmd!("jj", "new", rev).stdout_null().stderr_null().run()?;
        Ok(())
    }

    pub fn commit(message: &str) -> Result<()> {
        cmd!("jj", "commit", "-m", message)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn abandon(revset: &str) -> Result<()> {
        cmd!("jj", "abandon", revset)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }
}

/// Rebase operations
pub mod rebase {
    use duct::cmd;
    use eyre::Result;

    pub fn single(source: &str, dest: &str) -> Result<()> {
        cmd!("jj", "rebase", "-r", source, "-A", dest)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn single_with_next(source: &str, dest: &str, next: &str) -> Result<()> {
        cmd!("jj", "rebase", "-r", source, "-A", dest, "-B", next)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn with_descendants(source: &str, dest: &str) -> Result<()> {
        cmd!("jj", "rebase", "-s", source, "-A", dest)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    pub fn with_descendants_and_next(source: &str, dest: &str, next: &str) -> Result<()> {
        cmd!("jj", "rebase", "-s", source, "-A", dest, "-B", next)
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    /// Rebase single commit onto trunk()
    pub fn single_onto_trunk(source: &str) -> Result<()> {
        cmd!("jj", "rebase", "-r", source, "-d", "trunk()", "--skip-emptied")
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
    }

    /// Rebase commit with descendants onto trunk()
    pub fn with_descendants_onto_trunk(source: &str) -> Result<()> {
        cmd!("jj", "rebase", "-s", source, "-d", "trunk()", "--skip-emptied")
            .stdout_null()
            .stderr_capture()
            .run()?;
        Ok(())
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
    let output = cmd!("jj", "log", "-r", revset, "--no-graph", "-T", "change_id", "--limit", "1")
        .stdout_capture()
        .stderr_null()
        .read()?;
    Ok(!output.trim().is_empty())
}

/// Get the first child of a revision (if any)
pub fn get_first_child(rev: &str) -> Result<Option<String>> {
    let revset = format!("children({rev})");
    let output = cmd!("jj", "log", "-r", revset, "--no-graph", "-T", "change_id", "--limit", "1")
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
