mod diff_pane;
mod layout;
mod overlays;
mod status_bar;
mod tree_pane;

use super::app::App;
use super::vm::TreeRowVm;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub(super) use diff_pane::{render_diff, render_diff_pane};
#[cfg(test)]
use layout::PaneContent;
use layout::{pane_plan, panes_for, render_pane};
pub(crate) use tree_pane::format_bookmarks_truncated;
pub(super) use tree_pane::render_tree_with_vms;

/// Render with pre-built view models (avoids rebuilding when caller already has them)
pub fn render_with_vms(frame: &mut Frame, app: &App, vms: &[TreeRowVm]) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::state::{DiffLine, DiffLineKind, DiffState, ModeState, StyledSpan};
    use crate::cmd::jj_tui::test_support::{make_node, make_tree};
    use crate::cmd::jj_tui::vm::build_tree_view;
    use ratatui::{Terminal, backend::TestBackend, style::Color};
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    fn buffer_to_string(buf: &ratatui::buffer::Buffer) -> String {
        let mut output = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                output.push_str(buf[(x, y)].symbol());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn test_split_view_renders_tree_and_diff_when_viewing_diff() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);

        let diff = DiffState {
            lines: vec![DiffLine {
                spans: vec![StyledSpan {
                    text: "diff content".to_string(),
                    fg: Color::Reset,
                }],
                kind: DiffLineKind::Context,
            }],
            scroll_offset: 0,
            rev: "aaaa".to_string(),
        };

        let app = App {
            tree,
            mode: ModeState::ViewingDiff(diff),
            should_quit: false,
            split_view: true,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            repo_path: std::env::current_dir().unwrap_or_default(),
            detail_hydrator: None,
            detail_generation: 0,
        };

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("terminal init");

        let vms = build_tree_view(&app, 80);
        terminal
            .draw(|frame| render_with_vms(frame, &app, &vms))
            .expect("terminal draw");

        let screen = buffer_to_string(terminal.backend().buffer());
        assert!(
            screen.contains("jj tree"),
            "expected tree pane title in screen; got:\n{screen}"
        );
        assert!(
            screen.contains("Diff:"),
            "expected diff pane title in screen; got:\n{screen}"
        );
    }

    #[test]
    fn test_pane_plan_no_secondary_viewing_diff() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let diff = DiffState {
            lines: vec![],
            scroll_offset: 0,
            rev: "aaaa".to_string(),
        };
        let app = App {
            tree,
            mode: ModeState::ViewingDiff(diff),
            should_quit: false,
            split_view: false,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            repo_path: std::env::current_dir().unwrap_or_default(),
            detail_hydrator: None,
            detail_generation: 0,
        };

        let plan = pane_plan(&app, false);
        assert!(matches!(plan.primary, PaneContent::Diff(_)));
        assert!(plan.secondary.is_none());
    }

    #[test]
    fn test_pane_plan_with_secondary_viewing_diff() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let diff = DiffState {
            lines: vec![],
            scroll_offset: 0,
            rev: "aaaa".to_string(),
        };
        let app = App {
            tree,
            mode: ModeState::ViewingDiff(diff),
            should_quit: false,
            split_view: true,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            repo_path: std::env::current_dir().unwrap_or_default(),
            detail_hydrator: None,
            detail_generation: 0,
        };

        let plan = pane_plan(&app, true);
        assert!(matches!(plan.primary, PaneContent::Tree));
        assert!(matches!(plan.secondary, Some(PaneContent::Diff(_))));
    }

    #[test]
    fn test_pane_plan_with_secondary_normal_mode() {
        let tree = make_tree(vec![make_node("aaaa", 0)]);
        let app = App {
            tree,
            mode: ModeState::Normal,
            should_quit: false,
            split_view: true,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            last_op: None,
            pending_key: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            repo_path: std::env::current_dir().unwrap_or_default(),
            detail_hydrator: None,
            detail_generation: 0,
        };

        let plan = pane_plan(&app, true);
        assert!(matches!(plan.primary, PaneContent::Tree));
        assert!(matches!(plan.secondary, Some(PaneContent::DiffPlaceholder)));
    }
}
