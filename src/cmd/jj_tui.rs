mod app;
mod keybindings;
mod tree;
mod ui;

use eyre::Result;

pub fn run() -> Result<()> {
    let mut app = app::App::new()?;
    app.run()
}
