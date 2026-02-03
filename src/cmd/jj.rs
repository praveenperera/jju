use crate::jj_lib_helpers::JjRepo;
use ahash::{HashMap, HashMapExt};
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
        if line.contains("[deleted]") {
            if let Some(bookmark) = line.split_whitespace().next() {
                println!("{}{}", "Deleting merged bookmark: ".dimmed(), bookmark);
                cmd!("jj", "bookmark", "delete", bookmark)
                    .stdout_null()
                    .stderr_null()
                    .run()
                    .wrap_err_with(|| format!("failed to delete bookmark {bookmark}"))?;
            }
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
    use std::collections::HashSet;

    let jj_repo = JjRepo::load(None)?;

    // get the working copy change_id
    let working_copy = jj_repo.working_copy_commit()?;
    let working_copy_id = jj_repo.shortest_change_id(&working_copy, 4)?;

    // build revset: base | descendants(roots(base..@)) | @::
    let base = from.as_deref().unwrap_or("trunk()");
    let revset = format!("{base} | descendants(roots({base}..@)) | @::");
    let commits = jj_repo.eval_revset(&revset)?;

    #[derive(Clone)]
    struct TreeCommit {
        rev: String,
        unique_len: usize,
        bookmarks: String,
        description: String,
        parent_revs: Vec<String>,
        is_working_copy: bool,
    }

    // build commit map with shortest change IDs
    let mut commit_map: HashMap<String, TreeCommit> = HashMap::new();

    for commit in &commits {
        let (rev, unique_len) = jj_repo.change_id_with_prefix_len(commit, 4)?;
        let bookmarks_vec = jj_repo.bookmarks_at(commit);
        let bookmarks = match bookmarks_vec.len() {
            0 => String::new(),
            1 => bookmarks_vec[0].clone(),
            2 => {
                let both = format!("{} {}", bookmarks_vec[0], bookmarks_vec[1]);
                if both.len() <= 30 {
                    both
                } else {
                    format!("{} +1", bookmarks_vec[0])
                }
            }
            n => format!("{} +{}", bookmarks_vec[0], n - 1),
        };
        let description = JjRepo::description_first_line(commit);

        // get parent change IDs
        let parents = jj_repo.parent_commits(commit)?;
        let parent_revs: Vec<String> = parents
            .iter()
            .filter_map(|p| jj_repo.shortest_change_id(p, 4).ok())
            .collect();

        let is_working_copy = rev == working_copy_id;

        commit_map.insert(
            rev.clone(),
            TreeCommit {
                rev,
                unique_len,
                bookmarks,
                description,
                parent_revs,
                is_working_copy,
            },
        );
    }

    if commit_map.is_empty() {
        return Ok(());
    }

    // build children map
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for commit in commit_map.values() {
        for parent in &commit.parent_revs {
            children_map
                .entry(parent.clone())
                .or_default()
                .push(commit.rev.clone());
        }
    }

    // get base change_id for root detection
    let base_id = jj_repo
        .eval_revset_single(base)
        .ok()
        .and_then(|c| jj_repo.shortest_change_id(&c, 4).ok());

    // find roots (commits whose parents aren't in our set, OR the base)
    let revs_in_set: HashSet<&str> = commit_map.keys().map(|s| s.as_str()).collect();
    let mut roots: Vec<String> = commit_map
        .values()
        .filter(|c| {
            // always include base as root
            if let Some(ref bid) = base_id {
                if c.rev == *bid {
                    return true;
                }
            }
            c.parent_revs
                .iter()
                .all(|p| !revs_in_set.contains(p.as_str()))
        })
        .map(|c| c.rev.clone())
        .collect();
    roots.sort();

    // highlight the unique prefix (matches jj's disambiguation context)
    let format_rev = |commit: &TreeCommit| -> String {
        let (prefix, suffix) = commit.rev.split_at(commit.unique_len.min(commit.rev.len()));
        format!("{}{}", prefix.purple(), suffix.dimmed())
    };

    // determine visibility for filtered mode
    let is_visible = |commit: &TreeCommit| -> bool {
        full || !commit.bookmarks.is_empty() || commit.is_working_copy
    };

    // count hidden commits between visible ancestors and a commit
    fn count_hidden_between(
        commit_map: &HashMap<String, TreeCommit>,
        children_map: &HashMap<String, Vec<String>>,
        from: &str,
        to: &str,
        is_visible_fn: &dyn Fn(&TreeCommit) -> bool,
    ) -> usize {
        // BFS from `from` to `to`, counting non-visible commits in between
        let mut count = 0;
        let mut current = from.to_string();

        while let Some(children) = children_map.get(&current) {
            // find the child that leads to `to`
            let next = children.iter().find(|c| {
                if *c == to {
                    return true;
                }
                // check if `to` is a descendant of this child
                let mut stack = vec![c.as_str()];
                let mut visited = HashSet::new();
                while let Some(n) = stack.pop() {
                    if n == to {
                        return true;
                    }
                    if visited.insert(n) {
                        if let Some(grandchildren) = children_map.get(n) {
                            stack.extend(grandchildren.iter().map(|s| s.as_str()));
                        }
                    }
                }
                false
            });

            match next {
                Some(n) if n == to => break,
                Some(n) => {
                    if let Some(c) = commit_map.get(n) {
                        if !is_visible_fn(c) {
                            count += 1;
                        }
                    }
                    current = n.clone();
                }
                None => break,
            }
        }
        count
    }

    // check if a rev or any of its descendants are visible
    fn has_visible_descendant(
        rev: &str,
        commit_map: &HashMap<String, TreeCommit>,
        children_map: &HashMap<String, Vec<String>>,
        is_visible_fn: &dyn Fn(&TreeCommit) -> bool,
    ) -> bool {
        if let Some(commit) = commit_map.get(rev) {
            if is_visible_fn(commit) {
                return true;
            }
        }
        if let Some(children) = children_map.get(rev) {
            for child in children {
                if has_visible_descendant(child, commit_map, children_map, is_visible_fn) {
                    return true;
                }
            }
        }
        false
    }

    // recursive tree printing
    #[allow(clippy::too_many_arguments)]
    fn print_subtree(
        rev: &str,
        commit_map: &HashMap<String, TreeCommit>,
        children_map: &HashMap<String, Vec<String>>,
        prefix: &str,
        is_last: bool,
        full: bool,
        hidden_count: usize,
        is_visible_fn: &dyn Fn(&TreeCommit) -> bool,
        format_rev_fn: &dyn Fn(&TreeCommit) -> String,
    ) {
        let commit = match commit_map.get(rev) {
            Some(c) => c,
            None => return,
        };

        let visible = is_visible_fn(commit);

        // get children with visible descendants
        let children: Vec<&String> = children_map
            .get(rev)
            .map(|c| {
                c.iter()
                    .filter(|child| {
                        has_visible_descendant(child, commit_map, children_map, is_visible_fn)
                    })
                    .collect()
            })
            .unwrap_or_default();

        // if not visible, pass through to children with accumulated hidden count
        if !visible {
            for (i, child) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                print_subtree(
                    child,
                    commit_map,
                    children_map,
                    prefix,
                    is_last && is_last_child,
                    full,
                    hidden_count + 1,
                    is_visible_fn,
                    format_rev_fn,
                );
            }
            return;
        }

        // print this commit
        let connector = if prefix.is_empty() {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };
        let colored_rev = format_rev_fn(commit);

        let count_str = if !full && hidden_count > 0 {
            format!(" +{hidden_count}")
        } else {
            String::new()
        };

        let at_marker = if commit.is_working_copy { "@ " } else { "" };

        // always show revision first, then bookmark
        let name = if commit.bookmarks.is_empty() {
            format!("{at_marker}({}){count_str}", colored_rev)
        } else {
            format!(
                "{at_marker}({}) {}{count_str}",
                colored_rev,
                commit.bookmarks.cyan()
            )
        };

        let desc = if commit.description.is_empty() {
            if commit.is_working_copy {
                "(working copy)".dimmed().to_string()
            } else {
                "(no description)".dimmed().to_string()
            }
        } else {
            commit.description.dimmed().to_string()
        };

        println!("{prefix}{connector}{name}  {desc}");

        // calculate new prefix for children
        let child_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        // print children
        for (i, child) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;

            // count hidden commits between this visible commit and the child
            let child_hidden = if full {
                0
            } else {
                count_hidden_between(commit_map, children_map, rev, child, is_visible_fn)
            };

            print_subtree(
                child,
                commit_map,
                children_map,
                &child_prefix,
                is_last_child,
                full,
                child_hidden,
                is_visible_fn,
                format_rev_fn,
            );
        }
    }

    // print each root as a separate tree
    for (i, root) in roots.iter().enumerate() {
        let is_last_root = i == roots.len() - 1;
        print_subtree(
            root,
            &commit_map,
            &children_map,
            "",
            is_last_root,
            full,
            0,
            &is_visible,
            &format_rev,
        );
    }

    Ok(())
}
