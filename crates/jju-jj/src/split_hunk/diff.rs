use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffLineKind {
    Context,
    Added,
    Removed,
}

impl DiffLineKind {
    pub(crate) fn is_added(self) -> bool {
        matches!(self, Self::Added)
    }

    pub(crate) fn is_removed(self) -> bool {
        matches!(self, Self::Removed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffLine {
    pub(crate) kind: DiffLineKind,
    pub(crate) content: String,
}

impl DiffLine {
    fn parse(line: &str) -> Option<Self> {
        let (kind, content) = if let Some(content) = line.strip_prefix('+') {
            (DiffLineKind::Added, content.to_string())
        } else if let Some(content) = line.strip_prefix('-') {
            (DiffLineKind::Removed, content.to_string())
        } else if let Some(content) = line.strip_prefix(' ') {
            (DiffLineKind::Context, content.to_string())
        } else {
            return None;
        };

        Some(Self { kind, content })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffHunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub(crate) fn new(
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
    ) -> Self {
        Self {
            old_start,
            old_count,
            new_start,
            new_count,
            lines: Vec::new(),
        }
    }

    fn parse_header(line: &str) -> Self {
        let (old_start, old_count, new_start, new_count) = parse_hunk_header(line);
        Self::new(old_start, old_count, new_start, new_count)
    }

    pub(crate) fn push_line(&mut self, line: DiffLine) {
        self.lines.push(line);
    }

    pub(crate) fn first_line(&self) -> usize {
        self.new_start
    }

    pub(crate) fn last_line(&self) -> usize {
        self.new_start + self.new_count.saturating_sub(1)
    }

    pub(crate) fn old_start(&self) -> usize {
        self.old_start
    }

    pub(crate) fn old_count(&self) -> usize {
        self.old_count
    }

    pub(crate) fn lines(&self) -> &[DiffLine] {
        &self.lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileDiff {
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

    fn push_hunk(&mut self, hunk: DiffHunk) {
        self.hunks.push(hunk);
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedDiff {
    files: Vec<FileDiff>,
}

impl ParsedDiff {
    pub(crate) fn empty() -> Self {
        Self { files: Vec::new() }
    }

    pub(crate) fn parse(diff_output: &str) -> Self {
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
                current_hunk = Some(DiffHunk::parse_header(line));
                continue;
            }

            if let Some(hunk) = &mut current_hunk
                && let Some(diff_line) = DiffLine::parse(line)
            {
                hunk.push_line(diff_line);
            }
        }

        flush_current_hunk(&mut current_file, &mut current_hunk);
        flush_current_file(&mut files, &mut current_file);

        Self { files }
    }

    pub(crate) fn filter_by_path(mut self, filter: Option<&str>) -> Self {
        if let Some(filter) = filter {
            self.files.retain(|file| file.path.contains(filter));
        }
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub(crate) fn files(&self) -> &[FileDiff] {
        &self.files
    }
}

fn parse_file_path(line: &str) -> String {
    line.split_whitespace()
        .nth(3)
        .map(|part| part.trim_start_matches("b/"))
        .unwrap_or("")
        .to_string()
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
        file.push_hunk(hunk);
    }
}

fn flush_current_file(files: &mut Vec<FileDiff>, current_file: &mut Option<FileDiff>) {
    if let Some(file) = current_file.take() {
        files.push(file);
    }
}

#[cfg(test)]
mod tests {
    use super::{DiffLineKind, ParsedDiff};

    #[test]
    fn test_parse_diff_output_groups_files_and_hunks() {
        let parsed = ParsedDiff::parse(
            r#"diff --git a/src/lib.rs b/src/lib.rs
@@ -1,2 +1,3 @@
 line one
+line two
-line three
diff --git a/src/main.rs b/src/main.rs
@@ -3 +3 @@
-before
+after
"#,
        );

        assert_eq!(parsed.files().len(), 2);
        assert_eq!(parsed.files()[0].path(), "src/lib.rs");
        assert_eq!(parsed.files()[0].hunks().len(), 1);
        assert_eq!(parsed.files()[0].hunks()[0].first_line(), 1);
        assert_eq!(
            parsed.files()[0].hunks()[0].lines()[1].kind,
            DiffLineKind::Added
        );
        assert_eq!(parsed.files()[1].path(), "src/main.rs");
    }
}
