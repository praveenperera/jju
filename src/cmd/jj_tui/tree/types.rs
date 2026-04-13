use crate::jj_lib_helpers::CommitDetails;
use ahash::HashSet;

pub struct TreeState {
    pub snapshot: TreeSnapshot,
    pub view: TreeViewState,
    pub projection: crate::cmd::jj_tui::tree::TreeProjection,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeLoadScope {
    Stack,
    Neighborhood,
}

#[derive(Clone, Debug)]
pub struct VisibleEntry {
    pub node_index: usize,
    pub visual_depth: usize,
    pub has_separator_before: bool,
    pub neighborhood: Option<NeighborhoodEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Tree,
    Neighborhood(NeighborhoodState),
}

#[derive(Clone, Debug)]
pub struct TreeViewState {
    pub cursor: usize,
    pub scroll_offset: usize,
    pub full_mode: bool,
    pub load_scope: TreeLoadScope,
    pub view_mode: ViewMode,
    pub expanded_entry: Option<usize>,
    pub selected: HashSet<usize>,
    pub selection_anchor: Option<usize>,
    pub focus_stack: Vec<usize>, // stack of node_indices for nested zoom
}

impl TreeViewState {
    pub(crate) fn new(load_scope: TreeLoadScope) -> Self {
        Self {
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            load_scope,
            view_mode: ViewMode::Tree,
            expanded_entry: None,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }
}

const NEIGHBORHOOD_MIN_LEVEL: usize = 0;
const NEIGHBORHOOD_MAX_LEVEL: usize = 6;
const NEIGHBORHOOD_BASE_ANCESTOR_LIMIT: usize = 4;
const NEIGHBORHOOD_BASE_PREVIEW_DEPTH_LIMIT: usize = 2;
const NEIGHBORHOOD_ANCESTOR_STEP: usize = 4;
const NEIGHBORHOOD_PREVIEW_DEPTH_STEP: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborhoodEntry {
    pub is_preview: bool,
    pub hidden_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeighborhoodExtent {
    Local(usize),
    FullTree,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborhoodState {
    pub anchor_change_id: String,
    pub history: Vec<String>,
    pub extent: NeighborhoodExtent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborhoodResize {
    NoChange,
    Reprojected,
    ScopeChanged,
}

impl NeighborhoodState {
    pub(in crate::cmd::jj_tui::tree) fn new(anchor_change_id: String) -> Self {
        Self {
            anchor_change_id,
            history: Vec::new(),
            extent: NeighborhoodExtent::Local(NEIGHBORHOOD_MIN_LEVEL),
        }
    }

    pub fn is_full_tree(&self) -> bool {
        matches!(self.extent, NeighborhoodExtent::FullTree)
    }

    pub fn local_level(&self) -> Option<usize> {
        match self.extent {
            NeighborhoodExtent::Local(level) => Some(level),
            NeighborhoodExtent::FullTree => None,
        }
    }

    pub fn ancestor_limit(&self) -> Option<usize> {
        self.local_level()
            .map(|level| NEIGHBORHOOD_BASE_ANCESTOR_LIMIT + level * NEIGHBORHOOD_ANCESTOR_STEP)
    }

    pub fn preview_depth_limit(&self) -> Option<usize> {
        self.local_level().map(|level| {
            NEIGHBORHOOD_BASE_PREVIEW_DEPTH_LIMIT + level * NEIGHBORHOOD_PREVIEW_DEPTH_STEP
        })
    }

    pub fn expand(&mut self) -> NeighborhoodResize {
        match self.extent {
            NeighborhoodExtent::Local(level) if level < NEIGHBORHOOD_MAX_LEVEL => {
                self.extent = NeighborhoodExtent::Local(level + 1);
                NeighborhoodResize::Reprojected
            }
            NeighborhoodExtent::Local(_) => {
                self.extent = NeighborhoodExtent::FullTree;
                NeighborhoodResize::ScopeChanged
            }
            NeighborhoodExtent::FullTree => NeighborhoodResize::NoChange,
        }
    }

    pub fn shrink(&mut self) -> NeighborhoodResize {
        match self.extent {
            NeighborhoodExtent::FullTree => {
                self.extent = NeighborhoodExtent::Local(NEIGHBORHOOD_MAX_LEVEL);
                NeighborhoodResize::ScopeChanged
            }
            NeighborhoodExtent::Local(level) if level > NEIGHBORHOOD_MIN_LEVEL => {
                self.extent = NeighborhoodExtent::Local(level - 1);
                NeighborhoodResize::Reprojected
            }
            NeighborhoodExtent::Local(_) => NeighborhoodResize::NoChange,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeSnapshot {
    pub nodes: Vec<TreeNode>,
    pub topology: crate::cmd::jj_tui::tree::TreeTopology,
}

impl TreeSnapshot {
    pub(in crate::cmd::jj_tui::tree) fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            topology: crate::cmd::jj_tui::tree::TreeTopology::default(),
        }
    }

    pub(crate) fn from_nodes(nodes: Vec<TreeNode>) -> Self {
        let topology = crate::cmd::jj_tui::tree::TreeTopology::from_nodes(&nodes);
        Self { nodes, topology }
    }
}
