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

pub use app::AppOptions;

pub fn run() -> Result<()> {
    run_with_options(AppOptions::default())
}

pub fn run_with_options(options: AppOptions) -> Result<()> {
    let mut app = app::App::new(options)?;
    app.run()
}
