use crate::jj_lib_helpers::JjRepo;
use clap::{Parser, Subcommand};
use colored::Colorize;
use duct::cmd;
use eyre::{Context as _, Result};
use log::debug;
use std::ffi::OsString;
use std::io::Write;

#[derive(Debug, Clone, Parser)]
#[command(name = "jju", author, version, about, styles = crate::cli::get_styles())]
pub struct Jj {
    #[command(subcommand)]
    pub subcommand: Option<JjCmd>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum JjCmd {
    /// Sync the current stack with remote trunk (master/main/trunk)
    #[command(visible_alias = "ss")]
    StackSync {
        /// Push the first bookmark after syncing
        #[arg(short, long)]
        push: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Display the current stack as a tree
    #[command(visible_alias = "t")]
    Tree {
        /// Show all commits, including those without bookmarks
        #[arg(short, long)]
        full: bool,

        /// Base revision to start the tree from (default: trunk())
        #[arg(long)]
        from: Option<String>,
    },
}

pub fn run(args: &[OsString]) -> Result<()> {
    debug!("jj args: {args:?}");
    let flags = Jj::parse_from(args);
    run_with_flags(flags)
}

pub fn run_with_flags(flags: Jj) -> Result<()> {
    match flags.subcommand {
        None => crate::cmd::jj_tui::run(),
        Some(JjCmd::StackSync { push, force }) => stack_sync(push, force),
        Some(JjCmd::Tree { full, from }) => tree(full, from),
    }
}

/// Detect the trunk branch name (master, main, or trunk)
fn detect_trunk_branch() -> Result<String> {
    let output = cmd!(
        "jj",
        "log",
        "-r",
        "trunk()",
        "--no-graph",
        "-T",
        "local_bookmarks",
        "--limit",
        "1"
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .wrap_err("failed to detect trunk branch")?;

    let trunk = output
        .split_whitespace()
        .next()
        .unwrap_or("master")
        .to_string();

    Ok(trunk)
}

fn stack_sync(push: bool, force: bool) -> Result<()> {
    println!("{}", "Fetching from remote...".dimmed());
    cmd!("jj", "git", "fetch")
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err("failed to fetch")?;

    // detect and sync trunk bookmark
    let trunk = detect_trunk_branch()?;
    println!("{}{}", "Syncing ".dimmed(), trunk);
    cmd!(
        "jj",
        "bookmark",
        "set",
        &trunk,
        "-r",
        format!("{trunk}@origin")
    )
    .stdout_null()
    .stderr_null()
    .run()
    .wrap_err("failed to sync trunk bookmark")?;

    // find the root(s) of the stack
    let roots_revset = format!("roots({trunk}..@)");
    let roots_output = cmd!(
        "jj",
        "log",
        "-r",
        &roots_revset,
        "--no-graph",
        "-T",
        r#"change_id.short() ++ "\n""#
    )
    .stdout_capture()
    .stderr_capture()
    .read()
    .wrap_err("failed to find stack roots")?;

    let roots: Vec<&str> = roots_output.lines().filter(|l| !l.is_empty()).collect();
    debug!("stack roots: {:?}", roots);

    if roots.is_empty() {
        println!(
            "{}{}{}",
            "No commits after ".dimmed(),
            trunk,
            ", nothing to rebase".dimmed()
        );
        return Ok(());
    }

    // show confirmation unless --force
    if !force {
        println!(
            "Will rebase the following commits on top of {}:",
            trunk.cyan()
        );
        for root in &roots {
            let desc = cmd!(
                "jj",
                "log",
                "-r",
                root,
                "--no-graph",
                "-T",
                "description.first_line()"
            )
            .stdout_capture()
            .stderr_capture()
            .read()
            .unwrap_or_default();
            println!("  {}  {}", root.purple(), desc.dimmed());
            println!(
                "  {}",
                format!("jj rebase --source (-s) {root} --onto (-o) {trunk} --skip-emptied")
                    .dimmed()
            );
        }
        print!("Continue? [y/N] ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted".yellow());
            return Ok(());
        }
    }

    // rebase from each root (usually just one)
    // --skip-emptied handles merged commits by abandoning ones that became empty
    for root in &roots {
        println!("{}{}...", "Rebasing stack from ".dimmed(), root);
        cmd!(
            "jj",
            "rebase",
            "--source",
            root,
            "--onto",
            &trunk,
            "--skip-emptied"
        )
        .stdout_null()
        .stderr_null()
        .run()
        .wrap_err_with(|| format!("failed to rebase from {root}"))?;
    }

    // clean up bookmarks marked as deleted on remote (after rebase so --skip-emptied can work)
    let tracked = cmd!("jj", "bookmark", "list", "--tracked")
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to list tracked bookmarks")?;

    for line in tracked.lines() {
        if line.contains("[deleted]")
            && let Some(bookmark) = line.split_whitespace().next()
        {
            println!("{}{}", "Deleting merged bookmark: ".dimmed(), bookmark);
            cmd!("jj", "bookmark", "delete", bookmark)
                .stdout_null()
                .stderr_null()
                .run()
                .wrap_err_with(|| format!("failed to delete bookmark {bookmark}"))?;
        }
    }

    if push {
        // find and push the first bookmark in the rebased stack
        let revset = format!("({trunk}..@) & bookmarks()");
        let template = r#"bookmarks ++ "\n""#;
        let output = cmd!(
            "jj",
            "log",
            "-r",
            &revset,
            "--reversed",
            "--no-graph",
            "-T",
            template,
            "--limit",
            "1"
        )
        .stdout_capture()
        .stderr_capture()
        .read()
        .wrap_err("failed to get bookmarks")?;

        if let Some(bookmark) = output.lines().find(|l| !l.is_empty()) {
            let bookmark = bookmark.trim();
            println!("{}{}...", "Pushing ".dimmed(), bookmark);
            cmd!("jj", "git", "push", "--bookmark", bookmark)
                .stdout_null()
                .stderr_null()
                .run()
                .wrap_err("failed to push")?;
        } else {
            println!("{}", "No bookmarks found to push".dimmed());
        }
    }

    println!("{}", "Stack sync complete".green());
    Ok(())
}

fn tree(full: bool, from: Option<String>) -> Result<()> {
    use crate::cmd::jj_tui::{cli_tree, tree::TreeState};

    let jj_repo = JjRepo::load(None)?;
    let base = from.as_deref().unwrap_or("trunk()");
    let tree = TreeState::load_with_base(&jj_repo, base)?;
    cli_tree::print_tree(&tree, full);
    Ok(())
}
