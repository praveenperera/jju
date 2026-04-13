use super::SplitHunkOptions;
use super::diff::{DiffHunk, ParsedDiff};
use super::plan::SplitHunkPlan;
use eyre::{Result, WrapErr, eyre};
use jju_core::split_hunk::{LineRange, SplitSelectionPlan};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedHunk {
    pub(crate) file_index: usize,
    pub(crate) hunk_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SplitHunkPlanner {
    selection: SplitSelectionPlan,
    pattern: Option<Regex>,
}

impl SplitHunkPlanner {
    pub(crate) fn from_options(options: &SplitHunkOptions) -> Result<Self> {
        let selection = SplitSelectionPlan {
            hunk_indices: options
                .hunks
                .as_deref()
                .map(parse_hunk_indices)
                .transpose()?,
            line_ranges: options
                .lines
                .as_deref()
                .map(parse_line_ranges)
                .transpose()?,
            pattern: options.pattern.clone(),
            invert: options.invert,
        };
        let pattern = selection
            .pattern
            .as_deref()
            .map(Regex::new)
            .transpose()
            .wrap_err("invalid --pattern regex")?;

        Ok(Self { selection, pattern })
    }

    pub(crate) fn build(self, diff: ParsedDiff) -> SplitHunkPlan {
        let mut selected = Vec::new();
        let mut global_index = 0;

        for (file_index, file) in diff.files().iter().enumerate() {
            for (hunk_index, hunk) in file.hunks().iter().enumerate() {
                let mut matches = self.matches(hunk, global_index);
                if self.selection.invert {
                    matches = !matches;
                }

                if matches {
                    selected.push(SelectedHunk {
                        file_index,
                        hunk_index,
                    });
                }

                global_index += 1;
            }
        }

        SplitHunkPlan::new(diff, selected)
    }

    fn matches(&self, hunk: &DiffHunk, global_index: usize) -> bool {
        if self.selection.matches_all() {
            return true;
        }

        if let Some(indices) = self.selection.hunk_indices.as_deref()
            && indices.contains(&global_index)
        {
            return true;
        }

        if let Some(ranges) = self.selection.line_ranges.as_deref()
            && hunk_overlaps_lines(hunk, ranges)
        {
            return true;
        }

        if let Some(pattern) = &self.pattern
            && hunk
                .lines()
                .iter()
                .any(|line| pattern.is_match(&line.content))
        {
            return true;
        }

        false
    }
}

fn parse_line_ranges(input: &str) -> Result<Vec<LineRange>> {
    let mut ranges = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut split = part.split('-');
            let start: usize = split
                .next()
                .ok_or_else(|| eyre!("invalid range: {part}"))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range start: {part}"))?;
            let end: usize = split
                .next()
                .ok_or_else(|| eyre!("invalid range: {part}"))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range end: {part}"))?;
            ranges.push(LineRange(start, end));
            continue;
        }

        let line: usize = part
            .parse()
            .wrap_err_with(|| format!("invalid line: {part}"))?;
        ranges.push(LineRange(line, line));
    }
    Ok(ranges)
}

fn parse_hunk_indices(input: &str) -> Result<Vec<usize>> {
    input
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .wrap_err_with(|| format!("invalid hunk index: {part}"))
        })
        .collect()
}

fn hunk_overlaps_lines(hunk: &DiffHunk, ranges: &[LineRange]) -> bool {
    let hunk_start = hunk.first_line();
    let hunk_end = hunk.last_line();
    ranges
        .iter()
        .any(|LineRange(start, end)| hunk_start <= *end && hunk_end >= *start)
}

#[cfg(test)]
mod tests {
    use super::{SplitHunkPlanner, parse_hunk_indices, parse_line_ranges};
    use crate::split_hunk::SplitHunkOptions;
    use crate::split_hunk::diff::ParsedDiff;

    fn options() -> SplitHunkOptions {
        SplitHunkOptions {
            message: None,
            revision: "@".to_string(),
            file_filter: None,
            lines: None,
            hunks: None,
            pattern: None,
            preview: false,
            invert: false,
            dry_run: false,
        }
    }

    fn parsed_diff() -> ParsedDiff {
        ParsedDiff::parse(
            r#"diff --git a/src/lib.rs b/src/lib.rs
@@ -1,1 +1,2 @@
 line one
+selected
@@ -10,1 +10,2 @@
 line ten
+other
"#,
        )
    }

    #[test]
    fn test_parse_line_ranges_supports_ranges_and_single_lines() {
        let ranges = parse_line_ranges("3-5,8").expect("parse line ranges");
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].0, 3);
        assert_eq!(ranges[0].1, 5);
        assert_eq!(ranges[1].0, 8);
        assert_eq!(ranges[1].1, 8);
    }

    #[test]
    fn test_parse_hunk_indices_supports_multiple_entries() {
        assert_eq!(
            parse_hunk_indices("0,2,5").expect("parse hunk indices"),
            vec![0, 2, 5]
        );
    }

    #[test]
    fn test_build_selects_by_hunk_index() {
        let mut options = options();
        options.hunks = Some("1".to_string());

        let plan = SplitHunkPlanner::from_options(&options)
            .expect("planner")
            .build(parsed_diff());

        assert_eq!(plan.selected_count(), 1);
        assert_eq!(plan.selected_files()[0].selected_hunks, vec![1]);
    }

    #[test]
    fn test_build_selects_by_pattern() {
        let mut options = options();
        options.pattern = Some("selected".to_string());

        let plan = SplitHunkPlanner::from_options(&options)
            .expect("planner")
            .build(parsed_diff());

        assert_eq!(plan.selected_count(), 1);
        assert_eq!(plan.selected_files()[0].selected_hunks, vec![0]);
    }

    #[test]
    fn test_build_inverts_matches() {
        let mut options = options();
        options.pattern = Some("selected".to_string());
        options.invert = true;

        let plan = SplitHunkPlanner::from_options(&options)
            .expect("planner")
            .build(parsed_diff());

        assert_eq!(plan.selected_count(), 1);
        assert_eq!(plan.selected_files()[0].selected_hunks, vec![1]);
    }
}
