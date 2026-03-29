mod apply;
mod execute;
mod parse;
mod select;

use colored::Colorize;
use duct::cmd;
use eyre::{Context as _, Result};
use regex::Regex;

pub(crate) struct SplitHunkCommand {
    options: SplitHunkOptions,
}

impl SplitHunkCommand {
    pub(crate) fn new(options: SplitHunkOptions) -> Self {
        Self { options }
    }

    pub(crate) fn run(self) -> Result<()> {
        SplitHunkWorkflow::new(self.options).run()
    }
}

pub(crate) struct SplitHunkOptions {
    pub(crate) message: Option<String>,
    pub(crate) revision: String,
    pub(crate) file_filter: Option<String>,
    pub(crate) lines: Option<String>,
    pub(crate) hunks: Option<String>,
    pub(crate) pattern: Option<String>,
    pub(crate) preview: bool,
    pub(crate) invert: bool,
    pub(crate) dry_run: bool,
}

impl SplitHunkOptions {
    fn commit_message(&self) -> Result<&str> {
        if self.preview {
            return Ok("");
        }

        self.message
            .as_deref()
            .ok_or_else(|| eyre::eyre!("--message is required unless using --preview"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffLineKind {
    Context,
    Added,
    Removed,
}

impl DiffLineKind {
    fn is_added(self) -> bool {
        matches!(self, Self::Added)
    }

    fn is_removed(self) -> bool {
        matches!(self, Self::Removed)
    }
}

#[derive(Debug, Clone)]
struct DiffLine {
    kind: DiffLineKind,
    content: String,
}

#[derive(Debug, Clone)]
struct DiffHunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffLine>,
}

impl DiffHunk {
    fn new(old_start: usize, old_count: usize, new_start: usize, new_count: usize) -> Self {
        Self {
            old_start,
            old_count,
            new_start,
            new_count,
            lines: Vec::new(),
        }
    }

    fn first_line(&self) -> usize {
        self.new_start
    }

    fn last_line(&self) -> usize {
        self.new_start + self.new_count.saturating_sub(1)
    }
}

#[derive(Debug, Clone)]
struct FileDiff {
    path: String,
    hunks: Vec<DiffHunk>,
}

impl FileDiff {
    fn new(path: String) -> Self {
        Self {
            path,
            hunks: Vec::new(),
        }
    }
}

struct SplitHunkWorkflow {
    options: SplitHunkOptions,
}

impl SplitHunkWorkflow {
    fn new(options: SplitHunkOptions) -> Self {
        Self { options }
    }

    fn run(self) -> Result<()> {
        let files = self.load_files()?;
        if files.is_empty() {
            return Ok(());
        }

        if self.options.preview {
            preview_hunks(&files);
            return Ok(());
        }

        let selection = self.parse_selection()?;
        let selected = select::select_hunks(&files, &selection);
        if selected.is_empty() {
            println!("{}", "No hunks matched selection criteria".yellow());
            return Ok(());
        }

        println!(
            "{} {} hunks",
            "Selected".green(),
            selected.len().to_string().cyan()
        );

        let new_contents = apply::apply_selected_hunks(&files, &selected, &self.options.revision)?;

        if self.options.dry_run {
            println!("\n{}", "Dry run - would commit:".yellow());
            for path in new_contents.keys() {
                println!("  {}", path.cyan());
            }
            return Ok(());
        }

        let message = self.options.commit_message()?;
        let original_contents = execute::collect_original_contents(&files, &self.options.revision);
        execute::execute_split_hunk(
            &self.options.revision,
            message,
            &new_contents,
            &original_contents,
        )?;

        println!("{} {}", "Created split commit:".green(), message.cyan());
        Ok(())
    }

    fn load_files(&self) -> Result<Vec<FileDiff>> {
        let diff_output = cmd!("jj", "diff", "-r", &self.options.revision, "--git")
            .stdout_capture()
            .stderr_capture()
            .read()
            .wrap_err("failed to get diff")?;

        if diff_output.is_empty() {
            println!("{}", "No changes in revision".yellow());
            return Ok(Vec::new());
        }

        let mut files = parse::parse_diff_output(&diff_output);
        if let Some(filter) = self.options.file_filter.as_deref() {
            files.retain(|file| file.path.contains(filter));
        }

        if files.is_empty() {
            println!("{}", "No matching files found".yellow());
        }

        Ok(files)
    }

    fn parse_selection(&self) -> Result<select::SplitSelection> {
        Ok(select::SplitSelection::new(
            self.options
                .hunks
                .as_deref()
                .map(parse::parse_hunk_indices)
                .transpose()?,
            self.options
                .lines
                .as_deref()
                .map(parse::parse_line_ranges)
                .transpose()?,
            self.options
                .pattern
                .as_deref()
                .map(Regex::new)
                .transpose()?,
            self.options.invert,
        ))
    }
}

fn preview_hunks(files: &[FileDiff]) {
    let mut global_idx = 0;
    for file in files {
        println!("\n{}", file.path.cyan().bold());
        for hunk in &file.hunks {
            let (label, color) = select::categorize_hunk(hunk);
            println!(
                "\n  {} {} (lines {}-{})",
                format!("[{}]", global_idx).white().bold(),
                label.color(color),
                hunk.first_line(),
                hunk.last_line()
            );
            for line in &hunk.lines {
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
            global_idx += 1;
        }
    }
}

fn read_file_lines_or_empty(revision: &str, path: &str) -> Vec<String> {
    cmd!("jj", "file", "show", "-r", revision, path)
        .stdout_capture()
        .stderr_capture()
        .read()
        .unwrap_or_default()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_files() -> Vec<FileDiff> {
        vec![
            FileDiff {
                path: "src/lib.rs".to_string(),
                hunks: vec![
                    DiffHunk {
                        old_start: 1,
                        old_count: 1,
                        new_start: 1,
                        new_count: 1,
                        lines: vec![DiffLine {
                            kind: DiffLineKind::Added,
                            content: "alpha".to_string(),
                        }],
                    },
                    DiffHunk {
                        old_start: 3,
                        old_count: 1,
                        new_start: 3,
                        new_count: 2,
                        lines: vec![
                            DiffLine {
                                kind: DiffLineKind::Context,
                                content: "ctx".to_string(),
                            },
                            DiffLine {
                                kind: DiffLineKind::Added,
                                content: "needle".to_string(),
                            },
                        ],
                    },
                ],
            },
            FileDiff {
                path: "src/main.rs".to_string(),
                hunks: vec![DiffHunk {
                    old_start: 5,
                    old_count: 1,
                    new_start: 5,
                    new_count: 1,
                    lines: vec![DiffLine {
                        kind: DiffLineKind::Added,
                        content: "omega".to_string(),
                    }],
                }],
            },
        ]
    }

    #[test]
    fn test_parse_line_ranges_supports_ranges_and_single_lines() {
        assert_eq!(
            parse::parse_line_ranges("10-12,14").unwrap(),
            vec![(10, 12), (14, 14)]
        );
    }

    #[test]
    fn test_select_hunks_with_invert_returns_non_matching_hunks() {
        let files = sample_files();
        let selection =
            select::SplitSelection::new(None, None, Some(Regex::new("needle").unwrap()), true);

        let selected = select::select_hunks(&files, &selection);

        assert_eq!(selected, vec![(0, 0), (1, 0)]);
    }

    #[test]
    fn test_apply_hunks_to_lines_applies_from_bottom_to_top() {
        let parent_lines = vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
            "four".to_string(),
        ];
        let hunks = vec![
            DiffHunk {
                old_start: 2,
                old_count: 1,
                new_start: 2,
                new_count: 1,
                lines: vec![DiffLine {
                    kind: DiffLineKind::Added,
                    content: "TWO".to_string(),
                }],
            },
            DiffHunk {
                old_start: 4,
                old_count: 1,
                new_start: 4,
                new_count: 2,
                lines: vec![
                    DiffLine {
                        kind: DiffLineKind::Context,
                        content: "four".to_string(),
                    },
                    DiffLine {
                        kind: DiffLineKind::Added,
                        content: "five".to_string(),
                    },
                ],
            },
        ];

        let result = apply::apply_hunks_to_lines(&parent_lines, &hunks, &[0, 1]);

        assert_eq!(result, vec!["one", "TWO", "three", "four", "five"]);
    }
}
