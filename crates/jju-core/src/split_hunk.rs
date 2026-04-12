#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange(pub usize, pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitSelectionPlan {
    pub hunk_indices: Option<Vec<usize>>,
    pub line_ranges: Option<Vec<LineRange>>,
    pub pattern: Option<String>,
    pub invert: bool,
}

impl SplitSelectionPlan {
    pub fn matches_all(&self) -> bool {
        self.hunk_indices.is_none() && self.line_ranges.is_none() && self.pattern.is_none()
    }
}
