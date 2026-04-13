use super::super::diff::DiffHunk;
use super::{ParsedDiff, SelectedHunk};
use jju_core::split_hunk::{LineRange, SplitSelectionPlan};
use regex::Regex;

pub(crate) fn build_selected_hunks(
    diff: &ParsedDiff,
    selection: &SplitSelectionPlan,
    pattern: Option<&Regex>,
) -> Vec<SelectedHunk> {
    let mut selected = Vec::new();
    let mut global_index = 0;

    for (file_index, file) in diff.files().iter().enumerate() {
        for (hunk_index, hunk) in file.hunks().iter().enumerate() {
            if matches_selection(selection, pattern, hunk, global_index) {
                selected.push(SelectedHunk {
                    file_index,
                    hunk_index,
                });
            }

            global_index += 1;
        }
    }

    selected
}

pub(crate) fn matches_selection(
    selection: &SplitSelectionPlan,
    pattern: Option<&Regex>,
    hunk: &DiffHunk,
    global_index: usize,
) -> bool {
    let mut matches = matches_base_selection(selection, pattern, hunk, global_index);
    if selection.invert {
        matches = !matches;
    }
    matches
}

fn matches_base_selection(
    selection: &SplitSelectionPlan,
    pattern: Option<&Regex>,
    hunk: &DiffHunk,
    global_index: usize,
) -> bool {
    if selection.matches_all() {
        return true;
    }

    if let Some(indices) = selection.hunk_indices.as_deref()
        && indices.contains(&global_index)
    {
        return true;
    }

    if let Some(ranges) = selection.line_ranges.as_deref()
        && hunk_overlaps_lines(hunk, ranges)
    {
        return true;
    }

    if let Some(pattern) = pattern
        && hunk
            .lines()
            .iter()
            .any(|line| pattern.is_match(&line.content))
    {
        return true;
    }

    false
}

fn hunk_overlaps_lines(hunk: &DiffHunk, ranges: &[LineRange]) -> bool {
    let hunk_start = hunk.first_line();
    let hunk_end = hunk.last_line();
    ranges
        .iter()
        .any(|LineRange(start, end)| hunk_start <= *end && hunk_end >= *start)
}
