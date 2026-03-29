use super::{DiffHunk, FileDiff};
use colored::Color;
use regex::Regex;

#[derive(Debug)]
pub(super) struct SplitSelection {
    pub hunk_indices: Option<Vec<usize>>,
    pub line_ranges: Option<Vec<(usize, usize)>>,
    pub pattern: Option<Regex>,
    pub invert: bool,
}

impl SplitSelection {
    pub fn new(
        hunk_indices: Option<Vec<usize>>,
        line_ranges: Option<Vec<(usize, usize)>>,
        pattern: Option<Regex>,
        invert: bool,
    ) -> Self {
        Self {
            hunk_indices,
            line_ranges,
            pattern,
            invert,
        }
    }
}

pub(super) fn categorize_hunk(hunk: &DiffHunk) -> (&'static str, Color) {
    let has_added = hunk.lines.iter().any(|line| line.kind.is_added());
    let has_removed = hunk.lines.iter().any(|line| line.kind.is_removed());

    match (has_added, has_removed) {
        (true, true) => ("modified", Color::Yellow),
        (true, false) => ("added", Color::Green),
        (false, true) => ("removed", Color::Red),
        (false, false) => ("context", Color::White),
    }
}

pub(super) fn select_hunks(files: &[FileDiff], selection: &SplitSelection) -> Vec<(usize, usize)> {
    let mut selected = Vec::new();
    let mut global_idx = 0;

    for (file_idx, file) in files.iter().enumerate() {
        for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
            let mut matches = matches_selection(hunk, global_idx, selection);
            if selection.invert {
                matches = !matches;
            }

            if matches {
                selected.push((file_idx, hunk_idx));
            }

            global_idx += 1;
        }
    }

    selected
}

fn matches_selection(hunk: &DiffHunk, global_idx: usize, selection: &SplitSelection) -> bool {
    if selection.hunk_indices.is_none()
        && selection.line_ranges.is_none()
        && selection.pattern.is_none()
    {
        return true;
    }

    if let Some(indices) = selection.hunk_indices.as_deref()
        && indices.contains(&global_idx)
    {
        return true;
    }

    if let Some(ranges) = selection.line_ranges.as_deref()
        && hunk_overlaps_lines(hunk, ranges)
    {
        return true;
    }

    if let Some(pattern) = selection.pattern.as_ref()
        && hunk_matches_pattern(hunk, pattern)
    {
        return true;
    }

    false
}

fn hunk_overlaps_lines(hunk: &DiffHunk, ranges: &[(usize, usize)]) -> bool {
    let hunk_start = hunk.first_line();
    let hunk_end = hunk.last_line();
    ranges
        .iter()
        .any(|(start, end)| hunk_start <= *end && hunk_end >= *start)
}

fn hunk_matches_pattern(hunk: &DiffHunk, pattern: &Regex) -> bool {
    hunk.lines
        .iter()
        .any(|line| pattern.is_match(&line.content))
}
