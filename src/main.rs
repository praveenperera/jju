mod cli;
mod cmd;
mod jj_lib_helpers;

use color_eyre::Result;
use std::ffi::OsString;

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let args: Vec<OsString> = std::env::args_os().collect();
    cmd::jj::run(&args)
}
