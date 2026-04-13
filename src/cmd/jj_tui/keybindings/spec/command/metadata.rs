use super::{BindingBehavior, CommandSpec};

pub(super) fn effective_prefix_title(spec: &CommandSpec) -> Option<&'static str> {
    match spec.behavior {
        BindingBehavior::PendingPrefix { title } => Some(title),
        BindingBehavior::Action(_) => spec.prefix_title,
    }
}
