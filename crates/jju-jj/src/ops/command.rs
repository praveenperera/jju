use duct::cmd;
use eyre::Result;

pub(super) fn run_with_stderr(expr: duct::Expression) -> Result<()> {
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

pub(super) fn capture_stdout(args: &[&str]) -> Result<String> {
    Ok(cmd("jj", args).stdout_capture().stderr_null().read()?)
}
