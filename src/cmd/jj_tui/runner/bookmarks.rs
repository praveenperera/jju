use super::{Effect, MessageKind, RunCtx, error, operations};

pub(super) fn handle(ctx: &mut RunCtx<'_>, effect: Effect) {
    match effect {
        Effect::RunBookmarkSet { name, rev } => {
            let (text, kind) = operations::run_bookmark_set(&name, &rev);
            ctx.set_status(text, kind);
        }
        Effect::RunBookmarkSetBackwards { name, rev } => {
            let (text, kind) = operations::run_bookmark_set_backwards(&name, &rev);
            ctx.set_status(text, kind);
        }
        Effect::RunBookmarkDelete { name } => {
            match crate::cmd::jj_tui::commands::bookmark::delete(&name) {
                Ok(_) => ctx.success(format!("Deleted bookmark '{name}'")),
                Err(error_value) => {
                    let details = format!("{error_value}");
                    ctx.set_status(
                        error::set_error_with_details("Delete bookmark failed", &details),
                        MessageKind::Error,
                    );
                }
            }
        }
        _ => unreachable!("unsupported bookmark effect: {effect:?}"),
    }
}
