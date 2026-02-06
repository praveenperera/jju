use std::{env, fs, process::Command};

use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::eyre::{bail, Result};

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build and install jju
    Release {
        /// Where to install the binary
        #[arg(value_enum)]
        place: Place,
    },
}

#[derive(Clone, ValueEnum)]
enum Place {
    /// Install to ~/.local/bin/jju
    Local,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Release { place } => release(place),
    }
}

fn release(place: Place) -> Result<()> {
    match place {
        Place::Local => release_local(),
    }
}

fn release_local() -> Result<()> {
    let workspace_root = workspace_root()?;

    // build release binary
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&workspace_root)
        .status()?;

    if !status.success() {
        bail!("cargo build --release failed");
    }

    // ensure ~/.local/bin exists
    let home = env::var("HOME")?;
    let bin_dir = format!("{home}/.local/bin");
    fs::create_dir_all(&bin_dir)?;

    // copy binary
    let src = workspace_root.join("target/release/jju");
    let dest = format!("{bin_dir}/jju");

    // unlink first so we can overwrite even if the binary is currently running (macOS ETXTBSY)
    let _ = fs::remove_file(&dest);
    fs::copy(&src, &dest)?;

    println!("Installed jju to {dest}");
    Ok(())
}

fn workspace_root() -> Result<std::path::PathBuf> {
    let output = Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()?;

    if !output.status.success() {
        bail!("Failed to locate workspace root");
    }

    let path = String::from_utf8(output.stdout)?;
    let cargo_toml = std::path::PathBuf::from(path.trim());

    Ok(cargo_toml
        .parent()
        .expect("Cargo.toml should have a parent directory")
        .to_path_buf())
}
