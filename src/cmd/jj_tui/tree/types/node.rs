use crate::jj_lib_helpers::CommitDetails;

#[derive(Clone, Debug)]
pub struct BookmarkInfo {
    pub name: String,
    pub is_diverged: bool,
}

/// Information about a divergent version of a commit
#[derive(Clone, Debug)]
pub struct DivergentVersion {
    pub commit_id: String,
    pub is_local: bool, // heuristic: has working copy or newest timestamp
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub change_id: String,
    pub unique_prefix_len: usize,
    pub commit_id: String,
    pub description: String,
    pub bookmarks: Vec<BookmarkInfo>,
    pub is_working_copy: bool,
    pub has_conflicts: bool,
    pub is_divergent: bool,
    pub divergent_versions: Vec<DivergentVersion>, // all versions if divergent
    pub parent_ids: Vec<String>,
    pub depth: usize,
    pub details: Option<CommitDetails>,
}

impl TreeNode {
    pub fn is_visible(&self, full_mode: bool) -> bool {
        full_mode || !self.bookmarks.is_empty() || self.is_working_copy
    }

    /// Get bookmark names as strings (for compatibility)
    pub fn bookmark_names(&self) -> Vec<String> {
        self.bookmarks
            .iter()
            .map(|bookmark| bookmark.name.clone())
            .collect()
    }

    /// Check if any bookmark has the given name
    pub fn has_bookmark(&self, name: &str) -> bool {
        self.bookmarks.iter().any(|bookmark| bookmark.name == name)
    }

    pub fn has_details(&self) -> bool {
        self.details.is_some()
    }
}
