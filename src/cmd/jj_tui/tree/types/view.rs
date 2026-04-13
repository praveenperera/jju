use super::neighborhood::{NeighborhoodEntry, NeighborhoodState};
use ahash::HashSet;

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
