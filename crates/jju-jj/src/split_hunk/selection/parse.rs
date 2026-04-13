use eyre::{Result, WrapErr, eyre};
use jju_core::split_hunk::LineRange;

pub(crate) fn parse_line_ranges(input: &str) -> Result<Vec<LineRange>> {
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

pub(crate) fn parse_hunk_indices(input: &str) -> Result<Vec<usize>> {
    input
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .wrap_err_with(|| format!("invalid hunk index: {part}"))
        })
        .collect()
}
