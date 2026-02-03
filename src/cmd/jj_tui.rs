mod action;
mod app;
mod commands;
mod controller;
mod effect;
mod engine;
mod handlers;
mod keybindings;
mod preview;
mod runner;
mod state;
mod theme;
mod tree;
mod ui;
mod vm;

use eyre::Result;

pub fn run() -> Result<()> {
    let mut app = app::App::new()?;
    app.run()
}
