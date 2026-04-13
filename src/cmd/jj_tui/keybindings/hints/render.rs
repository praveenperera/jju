use super::scenario::StatusHintContext;
use crate::cmd::jj_tui::keybindings::{
    DisplayKind, ModeId,
    catalog::{DynamicHintValue, HintSpec},
    display::{KeyFormat, first_key, first_key_any_pending, join_keys, keys_for_label},
};

pub(super) fn render_specs(ctx: &StatusHintContext, specs: &[HintSpec]) -> String {
    let segments = specs
        .iter()
        .map(|spec| render_spec(ctx, *spec))
        .collect::<Vec<_>>();
    join_segments(&segments)
}

pub(super) fn render_spec(ctx: &StatusHintContext, spec: HintSpec) -> String {
    match spec {
        HintSpec::Command { label, value } => kv(ctx.mode, label, value),
        HintSpec::CommandAnyPending {
            label,
            format,
            value,
        } => format!(
            "{}:{value}",
            key_for_hint_any_pending(ctx.mode, label, format)
        ),
        HintSpec::CommandPair { left, right, value } => {
            format!(
                "{}/{}:{value}",
                key_for_hint(ctx.mode, left),
                key_for_hint(ctx.mode, right)
            )
        }
        HintSpec::CommandPairAnyPending {
            left,
            right,
            format,
            value,
        } => format!(
            "{}/{}:{value}",
            key_for_hint_any_pending(ctx.mode, left, format),
            key_for_hint_any_pending(ctx.mode, right, format)
        ),
        HintSpec::LabelKeys { label, value } => {
            let keys = keys_for_label(ctx.mode, None, label, true, KeyFormat::Space);
            format!("{}:{value}", join_keys(&keys, "/"))
        }
        HintSpec::Literal(text) => text.to_string(),
        HintSpec::DynamicCommand { label, value } => {
            kv(ctx.mode, label, dynamic_hint_value(ctx, value))
        }
    }
}

fn dynamic_hint_value(ctx: &StatusHintContext, value: DynamicHintValue) -> &'static str {
    match value {
        DynamicHintValue::RebaseBranches => {
            if ctx.rebase_allow_branches.unwrap_or(false) {
                "inline"
            } else {
                "branch"
            }
        }
    }
}

fn key_for_hint(mode: ModeId, label: &str) -> String {
    first_key(mode, None, label, DisplayKind::Primary)
        .unwrap_or_else(|| "?".to_string())
        .split_whitespace()
        .last()
        .unwrap_or("?")
        .to_string()
}

fn key_for_hint_any_pending(mode: ModeId, label: &str, format: KeyFormat) -> String {
    first_key_any_pending(mode, label, DisplayKind::Primary, format)
        .unwrap_or_else(|| "?".to_string())
}

fn kv(mode: ModeId, label: &str, value: &str) -> String {
    format!("{}:{value}", key_for_hint(mode, label))
}

fn join_segments(segments: &[String]) -> String {
    segments.join("  ")
}
