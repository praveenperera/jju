mod conflicts;
mod event_loop;
mod input;
mod neighborhood;

use super::App;

impl App {
    pub fn run(&mut self) -> eyre::Result<()> {
        event_loop::run(self)
    }
}
