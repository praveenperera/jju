use super::super::App;
use crate::cmd::jj_tui::{ui, vm};
use eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

pub(super) fn run(app: &mut App) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run_loop(app, &mut terminal);
    ratatui::restore();
    result
}

fn run_loop(app: &mut App, terminal: &mut DefaultTerminal) -> Result<()> {
    while !app.should_quit {
        let size = terminal.size()?;
        let viewport_height = size.height.saturating_sub(3) as usize;
        let viewport_width = size.width.saturating_sub(2) as usize;

        app.apply_detail_updates();
        app.ensure_expanded_row_data();

        let vms = vm::build_tree_view(app, viewport_width);
        let cursor_vm = vms.get(app.tree.view.cursor);
        let cursor_height = cursor_vm.map_or(1, |vm| {
            vm.height + if vm.has_separator_before { 1 } else { 0 }
        });
        app.tree.update_scroll(viewport_height, cursor_height);

        terminal.draw(|frame| ui::render_with_vms(frame, app, &vms))?;

        if event::poll(std::time::Duration::from_millis(33))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key, viewport_height, terminal);
        }
    }

    Ok(())
}
