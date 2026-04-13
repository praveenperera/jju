use duct::cmd;
use eyre::Result;
use std::process::Command;

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
