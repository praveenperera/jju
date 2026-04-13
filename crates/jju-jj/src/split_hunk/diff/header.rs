use regex::Regex;
use std::sync::OnceLock;

pub(super) fn parse_hunk_header(line: &str) -> (usize, usize, usize, usize) {
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
