use super::super::app::App;
use super::super::state::{DiffState, ModeState};
use super::super::vm::TreeRowVm;
use super::{Frame, Rect, render_diff, render_diff_pane, render_tree_with_vms};

#[derive(Debug, Clone, Copy)]
pub(super) struct Panes {
    pub primary: Rect,
    pub secondary: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PaneContent<'a> {
    Tree,
    Diff(&'a DiffState),
    DiffPlaceholder,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PanePlan<'a> {
    pub primary: PaneContent<'a>,
    pub secondary: Option<PaneContent<'a>>,
}

pub(super) fn panes_for(area: Rect, split_view: bool) -> Panes {
    if split_view {
        let split = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Percentage(50),
            ])
            .split(area);
        Panes {
            primary: split[0],
            secondary: Some(split[1]),
        }
    } else {
        Panes {
            primary: area,
            secondary: None,
        }
    }
}

pub(super) fn pane_plan<'a>(app: &'a App, has_secondary: bool) -> PanePlan<'a> {
    if has_secondary {
        match &app.mode {
            ModeState::ViewingDiff(state) => PanePlan {
                primary: PaneContent::Tree,
                secondary: Some(PaneContent::Diff(state)),
            },
            _ => PanePlan {
                primary: PaneContent::Tree,
                secondary: Some(PaneContent::DiffPlaceholder),
            },
        }
    } else {
        match &app.mode {
            ModeState::ViewingDiff(state) => PanePlan {
                primary: PaneContent::Diff(state),
                secondary: None,
            },
            _ => PanePlan {
                primary: PaneContent::Tree,
                secondary: None,
            },
        }
    }
}

pub(super) fn render_pane(
    frame: &mut Frame,
    app: &App,
    vms: &[TreeRowVm],
    area: Rect,
    content: PaneContent<'_>,
) {
    match content {
        PaneContent::Tree => render_tree_with_vms(frame, app, area, vms),
        PaneContent::Diff(state) => render_diff(frame, state, area),
        PaneContent::DiffPlaceholder => render_diff_pane(frame, app, area),
    }
}
