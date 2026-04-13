mod bookmarks;
mod details;
mod row;

pub(crate) use self::bookmarks::format_bookmarks_truncated;
use self::details::render_commit_details_from_vm;
use self::row::render_row;
use crate::cmd::jj_tui::app::App;
use crate::cmd::jj_tui::vm::TreeRowVm;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub(crate) fn render_tree_with_vms(frame: &mut Frame, app: &App, area: Rect, vms: &[TreeRowVm]) {
    let block = Block::default()
        .title(" jj tree ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    if app.tree.visible_count() == 0 {
        let empty = Paragraph::new("No commits found").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    frame.render_widget(
        Paragraph::new(render_visible_lines(
            vms,
            app.tree.view.scroll_offset,
            inner.height as usize,
        )),
        inner,
    );
}

fn render_visible_lines(
    vms: &[TreeRowVm],
    scroll_offset: usize,
    viewport_height: usize,
) -> Vec<Line<'static>> {
    let mut line_count = 0;
    let mut lines = Vec::new();

    for vm in vms.iter().skip(scroll_offset) {
        if line_count >= viewport_height {
            break;
        }

        if vm.has_separator_before {
            lines.push(Line::default());
            line_count += 1;
            if line_count >= viewport_height {
                break;
            }
        }

        lines.push(render_row(vm));
        line_count += 1;

        if let Some(details) = vm.details.as_ref() {
            for detail_line in render_commit_details_from_vm(vm, details, Color::Yellow) {
                if line_count >= viewport_height {
                    break;
                }
                lines.push(detail_line);
                line_count += 1;
            }
        }
    }

    lines
}
