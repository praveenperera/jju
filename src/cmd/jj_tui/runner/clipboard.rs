use super::RunCtx;
use crate::cmd::jj_tui::effect::Effect;
use crate::jj_lib_helpers::JjRepo;
use eyre::{Result, WrapErr, bail};
use std::io::Write;
use std::process::{Command, Stdio};

pub(super) fn handle(ctx: &mut RunCtx<'_>, effect: Effect) {
    match effect {
        Effect::CopyToClipboard { value, success } => copy_value(ctx, &value, &success),
        Effect::CopyCommitMessageToClipboard {
            commit_id,
            fallback,
        } => {
            let value = load_commit_message(ctx, &commit_id).unwrap_or(fallback);
            copy_value(ctx, &value, "Copied commit message to clipboard");
        }
        _ => unreachable!("unsupported clipboard effect: {effect:?}"),
    }
}

fn copy_value(ctx: &mut RunCtx<'_>, value: &str, success: &str) {
    match copy_text(value) {
        Ok(()) => ctx.success(success),
        Err(error) => ctx.error(format!("Failed to copy to clipboard: {error}")),
    }
}

fn load_commit_message(ctx: &RunCtx<'_>, commit_id: &str) -> Result<String> {
    let repo = JjRepo::load(Some(ctx.repo_path))?;
    let commit = repo.commit_by_id_hex(commit_id)?;
    Ok(commit.description().to_string())
}

#[cfg(target_os = "macos")]
fn copy_text(text: &str) -> Result<()> {
    write_to_command("pbcopy", &[], text)
}

#[cfg(target_os = "windows")]
fn copy_text(text: &str) -> Result<()> {
    write_to_command("clip", &[], text)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn copy_text(text: &str) -> Result<()> {
    let mut last_error = None;
    for (program, args) in [
        ("wl-copy", &[][..]),
        ("xclip", &["-selection", "clipboard"][..]),
        ("xsel", &["--clipboard", "--input"][..]),
    ] {
        match write_to_command(program, args, text) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }

    if let Some(error) = last_error {
        Err(error)
    } else {
        bail!("no clipboard command found")
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
fn copy_text(_text: &str) -> Result<()> {
    bail!("clipboard is not supported on this platform")
}

fn write_to_command(program: &str, args: &[&str], text: &str) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .wrap_err_with(|| format!("failed to start {program}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .wrap_err_with(|| format!("failed to write to {program} stdin"))?;
    }

    let status = child
        .wait()
        .wrap_err_with(|| format!("failed to wait for {program}"))?;
    if status.success() {
        Ok(())
    } else {
        bail!("{program} exited with status {status}")
    }
}
