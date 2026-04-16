#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquashOperation {
    pub source_revs: Vec<String>,
    pub target_rev: String,
    pub op_before: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractiveOperation {
    EditDescription { rev: String },
    Squash(SquashOperation),
    Resolve { file: String },
}
