mod action;
mod app;
pub mod cli_tree;
mod commands;
mod controller;
mod effect;
mod engine;
mod handlers;
mod keybindings;
mod preview;
mod refresh;
mod runner;
mod state;
#[cfg(test)]
mod test_support;
mod theme;
pub mod tree;
mod ui;
mod vm;

use eyre::Result;

pub fn run() -> Result<()> {
    let mut app = app::App::new()?;
    app.run()
}
