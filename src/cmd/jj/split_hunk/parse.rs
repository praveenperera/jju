use super::{DiffHunk, DiffLine, DiffLineKind, FileDiff};
use eyre::{Context as _, Result};
use regex::Regex;
use std::sync::OnceLock;

pub(super) fn parse_diff_output(diff_output: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;

    for line in diff_output.lines() {
        if line.starts_with("diff --git ") {
            flush_current_hunk(&mut current_file, &mut current_hunk);
            flush_current_file(&mut files, &mut current_file);
            current_file = Some(FileDiff::new(parse_file_path(line)));
            continue;
        }

        if line.starts_with("@@ ") {
            flush_current_hunk(&mut current_file, &mut current_hunk);
            let (old_start, old_count, new_start, new_count) = parse_hunk_header(line);
            current_hunk = Some(DiffHunk::new(old_start, old_count, new_start, new_count));
            continue;
        }

        if let Some(hunk) = &mut current_hunk
            && let Some(diff_line) = parse_diff_line(line)
        {
            hunk.lines.push(diff_line);
        }
    }

    flush_current_hunk(&mut current_file, &mut current_hunk);
    flush_current_file(&mut files, &mut current_file);
    files
}

pub(super) fn parse_line_ranges(input: &str) -> Result<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut split = part.split('-');
            let start: usize = split
                .next()
                .ok_or_else(|| eyre::eyre!("invalid range: {}", part))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range start: {}", part))?;
            let end: usize = split
                .next()
                .ok_or_else(|| eyre::eyre!("invalid range: {}", part))?
                .trim()
                .parse()
                .wrap_err_with(|| format!("invalid range end: {}", part))?;
            ranges.push((start, end));
        } else {
            let line: usize = part
                .parse()
                .wrap_err_with(|| format!("invalid line: {}", part))?;
            ranges.push((line, line));
        }
    }
    Ok(ranges)
}

pub(super) fn parse_hunk_indices(input: &str) -> Result<Vec<usize>> {
    input
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .wrap_err_with(|| format!("invalid hunk index: {}", part))
        })
        .collect()
}

fn parse_file_path(line: &str) -> String {
    line.split_whitespace()
        .nth(3)
        .map(|part| part.trim_start_matches("b/"))
        .unwrap_or("")
        .to_string()
}

fn parse_diff_line(line: &str) -> Option<DiffLine> {
    let (kind, content) = if let Some(content) = line.strip_prefix('+') {
        (DiffLineKind::Added, content.to_string())
    } else if let Some(content) = line.strip_prefix('-') {
        (DiffLineKind::Removed, content.to_string())
    } else if let Some(content) = line.strip_prefix(' ') {
        (DiffLineKind::Context, content.to_string())
    } else {
        return None;
    };

    Some(DiffLine { kind, content })
}

fn parse_hunk_header(line: &str) -> (usize, usize, usize, usize) {
    static HUNK_HEADER_RE: OnceLock<Option<Regex>> = OnceLock::new();
    let Some(regex) = HUNK_HEADER_RE
        .get_or_init(|| Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").ok())
        .as_ref()
    else {
        return (1, 1, 1, 1);
    };

    let Some(captures) = regex.captures(line) else {
        return (1, 1, 1, 1);
    };

    let old_start = captures
        .get(1)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    let old_count = captures
        .get(2)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    let new_start = captures
        .get(3)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    let new_count = captures
        .get(4)
        .and_then(|matched| matched.as_str().parse().ok())
        .unwrap_or(1);
    (old_start, old_count, new_start, new_count)
}

fn flush_current_hunk(current_file: &mut Option<FileDiff>, current_hunk: &mut Option<DiffHunk>) {
    if let Some(hunk) = current_hunk.take()
        && let Some(file) = current_file
    {
        file.hunks.push(hunk);
    }
}

fn flush_current_file(files: &mut Vec<FileDiff>, current_file: &mut Option<FileDiff>) {
    if let Some(file) = current_file.take() {
        files.push(file);
    }
}
