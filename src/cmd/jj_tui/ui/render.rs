use super::layout::{pane_plan, panes_for, render_pane};
use super::overlays;
use super::status_bar;
use crate::cmd::jj_tui::{app::App, vm::TreeRowVm};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// Render with pre-built view models (avoids rebuilding when caller already has them)
pub(crate) fn render_with_vms(frame: &mut Frame, app: &App, vms: &[TreeRowVm]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let panes = panes_for(chunks[0], app.split_view);
    let plan = pane_plan(app, panes.secondary.is_some());

    render_pane(frame, app, vms, panes.primary, plan.primary);
    if let (Some(area), Some(content)) = (panes.secondary, plan.secondary) {
        render_pane(frame, app, vms, area, content);
    }

    status_bar::render_status_bar(frame, app, chunks[1]);
    overlays::render_overlays(frame, app);
}
