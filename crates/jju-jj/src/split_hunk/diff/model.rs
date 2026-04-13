use super::header::parse_hunk_header;

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
    pub(super) fn parse(line: &str) -> Option<Self> {
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

    pub(super) fn parse_header(line: &str) -> Self {
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
    pub(super) fn new(path: String) -> Self {
        Self {
            path,
            hunks: Vec::new(),
        }
    }

    pub(super) fn push_hunk(&mut self, hunk: DiffHunk) {
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

    pub(super) fn from_files(files: Vec<FileDiff>) -> Self {
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
