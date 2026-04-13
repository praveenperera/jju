use super::diff::{DiffHunk, DiffLineKind};
use super::plan::SplitHunkPlan;
use colored::{Color, Colorize};

pub(crate) fn preview_plan(plan: &SplitHunkPlan) {
    let mut global_index = 0;
    for file in plan.files() {
        println!("\n{}", file.path().cyan().bold());
        for hunk in file.hunks() {
            let (label, color) = categorize_hunk(hunk);
            println!(
                "\n  {} {} (lines {}-{})",
                format!("[{global_index}]").white().bold(),
                label.color(color),
                hunk.first_line(),
                hunk.last_line()
            );
            for line in hunk.lines() {
                let prefix = match line.kind {
                    DiffLineKind::Context => " ".white(),
                    DiffLineKind::Added => "+".green(),
                    DiffLineKind::Removed => "-".red(),
                };
                let content = match line.kind {
                    DiffLineKind::Context => line.content.white(),
                    DiffLineKind::Added => line.content.green(),
                    DiffLineKind::Removed => line.content.red(),
                };
                println!("    {}{}", prefix, content);
            }
            global_index += 1;
        }
    }
}

fn categorize_hunk(hunk: &DiffHunk) -> (&'static str, Color) {
    let has_added = hunk.lines().iter().any(|line| line.kind.is_added());
    let has_removed = hunk.lines().iter().any(|line| line.kind.is_removed());

    match (has_added, has_removed) {
        (true, true) => ("modified", Color::Yellow),
        (true, false) => ("added", Color::Green),
        (false, true) => ("removed", Color::Red),
        (false, false) => ("context", Color::White),
    }
}
