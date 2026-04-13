use super::{BindingBehavior, CommandSpec, KeySequence};
use crate::cmd::jj_tui::{
    action::Action,
    keybindings::{ActionTemplate, Binding, DisplayKind},
};
use eyre::{Result, eyre};

pub(super) fn compile_bindings(spec: &CommandSpec) -> Result<Vec<Binding>> {
    if spec.keys.is_empty() {
        return Err(eyre!(
            "binding `{}` in mode {:?} must define at least one key",
            spec.label,
            spec.mode
        ));
    }

    let mut bindings = Vec::with_capacity(spec.keys.len());
    for (index, key) in spec.keys.iter().copied().enumerate() {
        let (pending_prefix, pattern) = key.compile()?;
        bindings.push(Binding {
            mode: spec.mode,
            pending_prefix,
            key: pattern,
            action: compile_action(spec, key)?,
            display: if index == 0 {
                DisplayKind::Primary
            } else {
                DisplayKind::Alias
            },
            label: spec.label,
        });
    }
    Ok(bindings)
}

fn compile_action(spec: &CommandSpec, key: KeySequence) -> Result<ActionTemplate> {
    match &spec.behavior {
        BindingBehavior::Action(template) => Ok(template.clone()),
        BindingBehavior::PendingPrefix { .. } => key
            .pending_char()
            .map(|prefix| ActionTemplate::Fixed(Action::SetPendingKey(prefix)))
            .ok_or_else(|| {
                eyre!(
                    "pending-prefix binding `{}` in mode {:?} must use plain character keys",
                    spec.label,
                    spec.mode
                )
            }),
    }
}
