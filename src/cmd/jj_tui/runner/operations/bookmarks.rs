use crate::cmd::jj_tui::commands;
use crate::cmd::jj_tui::runner::error::set_error_with_details;
use crate::cmd::jj_tui::state::MessageKind;

pub(super) fn run_bookmark_set(name: &str, rev: &str) -> (String, MessageKind) {
    bookmark_result(
        commands::bookmark::set(name, rev),
        "Move bookmark failed",
        name,
        rev,
    )
}

pub(super) fn run_bookmark_set_backwards(name: &str, rev: &str) -> (String, MessageKind) {
    bookmark_result(
        commands::bookmark::set_allow_backwards(name, rev),
        "Move bookmark failed",
        name,
        rev,
    )
}

fn bookmark_result(
    result: eyre::Result<()>,
    error_prefix: &str,
    name: &str,
    rev: &str,
) -> (String, MessageKind) {
    match result {
        Ok(_) => {
            let short_rev = &rev[..8.min(rev.len())];
            (
                format!("Moved bookmark '{name}' to {short_rev}"),
                MessageKind::Success,
            )
        }
        Err(error) => (
            set_error_with_details(error_prefix, &error.to_string()),
            MessageKind::Error,
        ),
    }
}
