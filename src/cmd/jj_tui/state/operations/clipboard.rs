#[derive(Debug, Clone)]
pub struct ClipboardBranchOption {
    pub key: char,
    pub branch: String,
}

#[derive(Debug, Clone)]
pub struct ClipboardBranchSelectState {
    pub target_rev: String,
    pub options: Vec<ClipboardBranchOption>,
}

impl ClipboardBranchSelectState {
    pub fn branch_for_key(&self, key: char) -> Option<&str> {
        self.options
            .iter()
            .find(|option| option.key == key)
            .map(|option| option.branch.as_str())
    }
}
