use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    FileHeader,
    Hunk,
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub fg: Color,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub spans: Vec<StyledSpan>,
    pub kind: DiffLineKind,
}

#[derive(Debug, Clone)]
pub struct DiffState {
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub rev: String,
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}
