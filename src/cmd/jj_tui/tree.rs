mod load;
mod navigation;
mod neighborhood;
mod projection;
mod selection;
mod topology;
mod types;
mod visible;

use crate::jj_lib_helpers::{CommitDetails, JjRepo};
use eyre::Result;

pub use projection::TreeProjection;
pub use topology::TreeTopology;
pub use types::{
    BookmarkInfo, DivergentVersion, NeighborhoodAnchor, NeighborhoodState, TreeLoadScope, TreeNode,
    TreeSnapshot, TreeState, TreeViewState, ViewMode, VisibleEntry,
};

impl TreeState {
    pub fn load_with_base(jj_repo: &JjRepo, base: &str) -> Result<Self> {
        Self::load_with_scope(jj_repo, base, TreeLoadScope::Stack)
    }

    pub fn load_with_scope(
        jj_repo: &JjRepo,
        base: &str,
        load_scope: TreeLoadScope,
    ) -> Result<Self> {
        load::load_tree_state(jj_repo, base, load_scope)
    }

    fn empty(load_scope: TreeLoadScope) -> Self {
        Self::from_snapshot(TreeSnapshot::empty(), TreeViewState::new(load_scope))
    }

    fn from_nodes(nodes: Vec<TreeNode>, load_scope: TreeLoadScope) -> Self {
        Self::from_snapshot(
            TreeSnapshot::from_nodes(nodes),
            TreeViewState::new(load_scope),
        )
    }

    pub fn from_snapshot(snapshot: TreeSnapshot, view: TreeViewState) -> Self {
        let projection = TreeProjection::from_parts(&snapshot, &view, None);
        Self {
            snapshot,
            view,
            projection,
        }
    }

    pub fn visible_nodes(&self) -> impl Iterator<Item = &VisibleEntry> {
        self.projection.visible_entries.iter()
    }

    pub fn get_node(&self, entry: &VisibleEntry) -> &TreeNode {
        &self.snapshot.nodes[entry.node_index]
    }

    pub fn nodes(&self) -> &[TreeNode] {
        &self.snapshot.nodes
    }

    pub fn visible_entries(&self) -> &[VisibleEntry] {
        &self.projection.visible_entries
    }

    pub fn visible_count(&self) -> usize {
        self.projection.visible_entries.len()
    }

    pub fn current_entry(&self) -> Option<&VisibleEntry> {
        self.projection.visible_entries.get(self.view.cursor)
    }

    pub fn current_node(&self) -> Option<&TreeNode> {
        self.current_entry()
            .map(|entry| &self.snapshot.nodes[entry.node_index])
    }

    pub fn hydrate_details(&mut self, commit_id: &str, details: CommitDetails) -> bool {
        let Some(node) = self
            .snapshot
            .nodes
            .iter_mut()
            .find(|node| node.commit_id == commit_id)
        else {
            return false;
        };

        if node.details.is_some() {
            return false;
        }

        node.details = Some(details);
        true
    }

    pub fn toggle_full_mode(&mut self) {
        self.view.full_mode = !self.view.full_mode;
        self.recompute_projection();
    }

    fn recompute_projection(&mut self) {
        let current_entry_node_index = self.current_entry().map(|entry| entry.node_index);
        self.projection =
            TreeProjection::from_parts(&self.snapshot, &self.view, current_entry_node_index);

        if self.view.cursor >= self.visible_count() {
            self.view.cursor = self.visible_count().saturating_sub(1);
        }

        self.view.expanded_entry = None;
    }

    pub fn set_view_mode(&mut self, view_mode: ViewMode) {
        if matches!(view_mode, ViewMode::Neighborhood(..)) {
            self.view.focus_stack.clear();
        }
        self.view.view_mode = view_mode;
        self.recompute_projection();
    }
}
