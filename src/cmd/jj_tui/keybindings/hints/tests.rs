use super::{StatusHintContext, render::render_spec, scenario::scenario_for_context};
use crate::cmd::jj_tui::keybindings::{
    ModeId,
    catalog::{DynamicHintValue, HintScenario, HintSpec, command_id as cmd},
    display::KeyFormat,
};

fn ctx(mode: ModeId) -> StatusHintContext {
    StatusHintContext {
        mode,
        has_selection: false,
        has_focus: false,
        neighborhood_active: false,
        current_has_bookmark: false,
        rebase_allow_branches: None,
    }
}

#[test]
fn test_normal_scenario_precedence_prefers_selection() {
    let mut context = ctx(ModeId::Normal);
    context.has_selection = true;
    context.neighborhood_active = true;
    context.has_focus = true;
    context.current_has_bookmark = true;

    assert_eq!(
        scenario_for_context(&context),
        HintScenario::NormalSelection
    );
}

#[test]
fn test_normal_scenario_precedence_prefers_neighborhood_over_focus() {
    let mut context = ctx(ModeId::Normal);
    context.neighborhood_active = true;
    context.has_focus = true;
    context.current_has_bookmark = true;

    assert_eq!(
        scenario_for_context(&context),
        HintScenario::NormalNeighborhood
    );
}

#[test]
fn test_normal_scenario_precedence_prefers_focus_over_bookmarked() {
    let mut context = ctx(ModeId::Normal);
    context.has_focus = true;
    context.current_has_bookmark = true;

    assert_eq!(scenario_for_context(&context), HintScenario::NormalFocus);
}

#[test]
fn test_normal_scenario_precedence_prefers_bookmarked_over_default() {
    let mut context = ctx(ModeId::Normal);
    context.current_has_bookmark = true;

    assert_eq!(
        scenario_for_context(&context),
        HintScenario::NormalBookmarked
    );
}

#[test]
fn test_render_command_spec() {
    assert_eq!(
        render_spec(
            &ctx(ModeId::Normal),
            HintSpec::Command {
                label: cmd::DESC,
                value: "desc",
            }
        ),
        "D:desc"
    );
}

#[test]
fn test_render_command_any_pending_spec() {
    assert_eq!(
        render_spec(
            &ctx(ModeId::Normal),
            HintSpec::CommandAnyPending {
                label: cmd::NEIGHBORHOOD,
                format: KeyFormat::Concat,
                value: "full",
            }
        ),
        "zn:full"
    );
}

#[test]
fn test_render_command_pair_spec() {
    assert_eq!(
        render_spec(
            &ctx(ModeId::Selecting),
            HintSpec::CommandPair {
                left: cmd::DOWN,
                right: cmd::UP,
                value: "extend",
            }
        ),
        "j/k:extend"
    );
}

#[test]
fn test_render_command_pair_any_pending_spec() {
    assert_eq!(
        render_spec(
            &ctx(ModeId::Diff),
            HintSpec::CommandPairAnyPending {
                left: cmd::TOP,
                right: cmd::BOTTOM,
                format: KeyFormat::Concat,
                value: "top/bottom",
            }
        ),
        "zt/zb:top/bottom"
    );
}

#[test]
fn test_render_label_keys_spec() {
    assert_eq!(
        render_spec(
            &ctx(ModeId::Confirm),
            HintSpec::LabelKeys {
                label: cmd::YES,
                value: "yes",
            }
        ),
        "y/Enter:yes"
    );
}

#[test]
fn test_render_literal_spec() {
    assert_eq!(
        render_spec(
            &ctx(ModeId::BookmarkPicker),
            HintSpec::Literal("type:filter")
        ),
        "type:filter"
    );
}

#[test]
fn test_render_dynamic_command_spec() {
    let mut context = ctx(ModeId::Rebase);
    context.rebase_allow_branches = Some(true);

    assert_eq!(
        render_spec(
            &context,
            HintSpec::DynamicCommand {
                label: cmd::BRANCHES,
                value: DynamicHintValue::RebaseBranches,
            }
        ),
        "b:inline"
    );
}
