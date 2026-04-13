mod parse;
mod planner;
#[cfg(test)]
mod tests;

use super::SplitHunkOptions;
use super::diff::ParsedDiff;
use super::plan::SplitHunkPlan;
use eyre::{Result, WrapErr};
use jju_core::split_hunk::SplitSelectionPlan;
use regex::Regex;

pub(crate) use parse::{parse_hunk_indices, parse_line_ranges};

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
        SplitHunkPlan::new(
            diff.clone(),
            planner::build_selected_hunks(&diff, &self.selection, self.pattern.as_ref()),
        )
    }
}
