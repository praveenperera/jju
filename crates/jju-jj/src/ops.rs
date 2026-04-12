use duct::cmd;
use eyre::Result;
use std::process::Command;

fn run_with_stderr(expr: duct::Expression) -> Result<()> {
    let output = expr.stdout_null().stderr_capture().unchecked().run()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if stderr.is_empty() {
            eyre::bail!("command failed with exit code {:?}", output.status.code());
        } else {
            eyre::bail!("{stderr}");
        }
    }
    Ok(())
}

fn capture_stdout(args: &[&str]) -> Result<String> {
    Ok(cmd("jj", args).stdout_capture().stderr_null().read()?)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OperationOps;

impl OperationOps {
    pub fn current_op_id(self) -> Result<String> {
        let output = cmd!("jj", "op", "log", "--limit", "1", "-T", "id", "--no-graph")
            .stdout_capture()
            .stderr_null()
            .read()?;
        let op_id = output.trim().to_string();
        if op_id.is_empty() {
            eyre::bail!("jj op log returned empty operation ID");
        }
        Ok(op_id)
    }

    pub fn restore(self, op_id: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "op", "restore", op_id))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RevisionOps;

impl RevisionOps {
    pub fn edit(self, rev: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "edit", rev))
    }

    pub fn new_commit(self, rev: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "new", rev))
    }

    pub fn commit(self, message: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "commit", "-m", message))
    }

    pub fn abandon(self, revset: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "abandon", revset))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BookmarkOps;

impl BookmarkOps {
    pub fn set(self, name: &str, rev: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "bookmark", "set", name, "-r", rev))
    }

    pub fn set_allow_backwards(self, name: &str, rev: &str) -> Result<()> {
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

    pub fn delete(self, name: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "bookmark", "delete", name))
    }

    pub fn track(self, name: &str) -> Result<()> {
        let remote_ref = format!("{name}@origin");
        run_with_stderr(cmd!("jj", "bookmark", "track", &remote_ref))
    }
}

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

#[derive(Debug, Clone, Copy, Default)]
pub struct RebaseOps;

impl RebaseOps {
    pub fn single(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-r", source, "-A", dest))
    }

    pub fn with_descendants(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-s", source, "-A", dest))
    }

    pub fn single_fork(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-r", source, "-d", dest))
    }

    pub fn with_descendants_fork(self, source: &str, dest: &str) -> Result<()> {
        run_with_stderr(cmd!("jj", "rebase", "-s", source, "-d", dest))
    }

    pub fn single_onto_trunk(self, source: &str) -> Result<()> {
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

    pub fn with_descendants_onto_trunk(self, source: &str) -> Result<()> {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffOps;

impl DiffOps {
    pub fn get_diff(self, rev: &str) -> Result<String> {
        capture_stdout(&["diff", "--git", "-r", rev])
    }

    pub fn get_stats(self, change_id: &str) -> Result<String> {
        capture_stdout(&["diff", "--stat", "-r", change_id])
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConflictOps;

impl ConflictOps {
    pub fn has_conflicts(self) -> Result<bool> {
        let output = cmd!("jj", "log", "-r", "@", "-T", r#"if(conflict, "conflict")"#)
            .stdout_capture()
            .stderr_null()
            .read()?;
        Ok(output.contains("conflict"))
    }

    pub fn list_conflict_files(self) -> Result<Vec<String>> {
        let template = r#"conflict_files.map(|x| x ++ "\n").join("")"#;
        let output = cmd!("jj", "log", "-r", "@", "-T", template, "--no-graph")
            .stdout_capture()
            .stderr_null()
            .read()?;
        Ok(output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect())
    }

    pub fn resolve_file(self, file: &str) -> Result<()> {
        Command::new("jj").args(["resolve", file]).status()?;
        Ok(())
    }
}

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
