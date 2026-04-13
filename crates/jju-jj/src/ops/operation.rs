use super::command::run_with_stderr;
use duct::cmd;
use eyre::Result;

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
