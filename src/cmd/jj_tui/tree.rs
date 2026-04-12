mod load;
mod topology;
mod visible;

use crate::jj_lib_helpers::{CommitDetails, JjRepo};
use ahash::{HashMap, HashSet};
use eyre::Result;
pub use topology::TreeTopology;
use visible::{NeighborhoodFilter, VisibleOptions};

const NEIGHBORHOOD_MIN_LEVEL: usize = 0;
const NEIGHBORHOOD_MAX_LEVEL: usize = 6;
const NEIGHBORHOOD_BASE_ANCESTOR_LIMIT: usize = 4;
const NEIGHBORHOOD_BASE_DESCENDANT_LIMIT: usize = 2;
const NEIGHBORHOOD_BASE_SIBLING_DEPTH_LIMIT: usize = 2;
const NEIGHBORHOOD_ANCESTOR_STEP: usize = 4;
const NEIGHBORHOOD_DESCENDANT_STEP: usize = 2;
const NEIGHBORHOOD_SIBLING_STEP: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeLoadScope {
    Stack,
    Neighborhood,
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

#[derive(Clone, Debug)]
pub struct VisibleEntry {
    pub node_index: usize,
    pub visual_depth: usize,
    pub has_separator_before: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeighborhoodAnchor {
    FollowCursor,
    Fixed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborhoodState {
    pub anchor: NeighborhoodAnchor,
    pub level: usize,
}

impl NeighborhoodState {
    fn follow_cursor() -> Self {
        Self {
            anchor: NeighborhoodAnchor::FollowCursor,
            level: NEIGHBORHOOD_MIN_LEVEL,
        }
    }

    pub fn ancestor_limit(&self) -> usize {
        NEIGHBORHOOD_BASE_ANCESTOR_LIMIT + self.level * NEIGHBORHOOD_ANCESTOR_STEP
    }

    pub fn descendant_limit(&self) -> usize {
        NEIGHBORHOOD_BASE_DESCENDANT_LIMIT + self.level * NEIGHBORHOOD_DESCENDANT_STEP
    }

    pub fn sibling_depth_limit(&self) -> usize {
        NEIGHBORHOOD_BASE_SIBLING_DEPTH_LIMIT + self.level * NEIGHBORHOOD_SIBLING_STEP
    }

    pub fn expand(&mut self) -> bool {
        if self.level >= NEIGHBORHOOD_MAX_LEVEL {
            return false;
        }
        self.level += 1;
        true
    }

    pub fn shrink(&mut self) -> bool {
        if self.level == NEIGHBORHOOD_MIN_LEVEL {
            return false;
        }
        self.level -= 1;
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Tree,
    Neighborhood(NeighborhoodState),
}

#[derive(Clone, Debug)]
pub struct TreeSnapshot {
    pub nodes: Vec<TreeNode>,
    pub topology: TreeTopology,
}

impl TreeSnapshot {
    fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            topology: TreeTopology::default(),
        }
    }

    pub(crate) fn from_nodes(nodes: Vec<TreeNode>) -> Self {
        let topology = TreeTopology::from_nodes(&nodes);
        Self { nodes, topology }
    }
}

#[derive(Clone, Debug)]
pub struct TreeProjection {
    pub visible_entries: Vec<VisibleEntry>,
}

impl TreeProjection {
    fn from_parts(
        snapshot: &TreeSnapshot,
        view: &TreeViewState,
        current_entry_node_index: Option<usize>,
    ) -> Self {
        let neighborhood_anchor =
            resolve_neighborhood_anchor_index(snapshot, view, current_entry_node_index);
        let visible_entries = visible::compute_visible_entries(
            &snapshot.nodes,
            &snapshot.topology,
            VisibleOptions {
                full_mode: view.full_mode,
                focused_root: focused_root_index(view),
                neighborhood: neighborhood_anchor.and_then(|anchor_index| {
                    neighborhood_state(&view.view_mode).map(|state| NeighborhoodFilter {
                        anchor_index,
                        ancestor_limit: state.ancestor_limit(),
                        descendant_limit: state.descendant_limit(),
                        sibling_depth_limit: state.sibling_depth_limit(),
                    })
                }),
            },
        );
        Self { visible_entries }
    }
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

pub struct TreeState {
    pub snapshot: TreeSnapshot,
    pub view: TreeViewState,
    pub projection: TreeProjection,
}

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

    pub fn move_cursor_up(&mut self) {
        if self.view.cursor > 0 {
            self.view.cursor -= 1;
        }
        self.sync_neighborhood_to_cursor();
    }

    pub fn move_cursor_down(&mut self) {
        if self.view.cursor + 1 < self.visible_count() {
            self.view.cursor += 1;
        }
        self.sync_neighborhood_to_cursor();
    }

    pub fn move_cursor_top(&mut self) {
        self.view.cursor = 0;
        self.view.scroll_offset = 0;
        self.sync_neighborhood_to_cursor();
    }

    pub fn move_cursor_bottom(&mut self) {
        let count = self.visible_count();
        if count > 0 {
            self.view.cursor = count - 1;
        }
        self.sync_neighborhood_to_cursor();
    }

    pub fn jump_to_working_copy(&mut self) {
        for (index, entry) in self.projection.visible_entries.iter().enumerate() {
            if self.snapshot.nodes[entry.node_index].is_working_copy {
                self.view.cursor = index;
                self.sync_neighborhood_to_cursor();
                return;
            }
        }
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

    /// Toggle focus on the current node
    pub fn toggle_focus(&mut self) {
        if self.is_neighborhood_mode() {
            return;
        }

        let Some(entry) = self.current_entry() else {
            return;
        };
        let current_node_idx = entry.node_index;

        if self.view.focus_stack.last() == Some(&current_node_idx) && self.view.cursor == 0 {
            self.unfocus();
            return;
        }

        self.focus_on(current_node_idx);
    }

    /// Focus on a specific node (zoom in), pushing to the focus stack
    pub fn focus_on(&mut self, node_index: usize) {
        if self.is_neighborhood_mode() {
            return;
        }

        self.view.focus_stack.push(node_index);
        self.recompute_projection();
        self.view.cursor = 0;
        self.view.scroll_offset = 0;
    }

    /// Unfocus one level (zoom out), popping from the focus stack
    pub fn unfocus(&mut self) {
        let popped_change_id = self.view.focus_stack.pop().and_then(|index| {
            self.snapshot
                .nodes
                .get(index)
                .map(|node| node.change_id.clone())
        });

        self.recompute_projection();

        if let Some(change_id) = popped_change_id
            && let Some(index) = self
                .projection
                .visible_entries
                .iter()
                .position(|entry| self.snapshot.nodes[entry.node_index].change_id == change_id)
        {
            self.view.cursor = index;
        }
    }

    /// Returns true if the tree is currently focused (zoomed)
    pub fn is_focused(&self) -> bool {
        !self.view.focus_stack.is_empty()
    }

    pub fn is_neighborhood_mode(&self) -> bool {
        matches!(&self.view.view_mode, ViewMode::Neighborhood(..))
    }

    pub fn is_neighborhood_following_cursor(&self) -> bool {
        matches!(
            &self.view.view_mode,
            ViewMode::Neighborhood(NeighborhoodState {
                anchor: NeighborhoodAnchor::FollowCursor,
                ..
            })
        )
    }

    pub fn enable_neighborhood(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        self.view.load_scope = TreeLoadScope::Neighborhood;
        self.view.focus_stack.clear();
        self.set_view_mode(ViewMode::Neighborhood(NeighborhoodState::follow_cursor()));
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn disable_neighborhood(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        self.view.load_scope = TreeLoadScope::Stack;
        self.set_view_mode(ViewMode::Tree);
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn toggle_neighborhood(&mut self) {
        if self.is_neighborhood_mode() {
            self.disable_neighborhood();
        } else {
            self.enable_neighborhood();
        }
    }

    pub fn freeze_neighborhood_anchor(&mut self) {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return;
        };
        if let ViewMode::Neighborhood(state) = &mut self.view.view_mode {
            state.anchor = NeighborhoodAnchor::Fixed(change_id);
        }
    }

    pub fn resume_neighborhood_follow_cursor(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        if let ViewMode::Neighborhood(state) = &mut self.view.view_mode {
            state.anchor = NeighborhoodAnchor::FollowCursor;
        } else {
            return;
        }

        self.recompute_projection();
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn neighborhood_state(&self) -> Option<&NeighborhoodState> {
        match &self.view.view_mode {
            ViewMode::Neighborhood(state) => Some(state),
            ViewMode::Tree => None,
        }
    }

    pub fn expand_neighborhood(&mut self) -> bool {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return false;
        };
        if !state.expand() {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        true
    }

    pub fn shrink_neighborhood(&mut self) -> bool {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return false;
        };
        if !state.shrink() {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        true
    }

    pub fn set_view_mode(&mut self, view_mode: ViewMode) {
        if matches!(view_mode, ViewMode::Neighborhood(..)) {
            self.view.focus_stack.clear();
        }
        self.view.view_mode = view_mode;
        self.recompute_projection();
    }

    /// Returns the current focus depth (number of zoom levels)
    pub fn focus_depth(&self) -> usize {
        self.view.focus_stack.len()
    }

    /// Get the currently focused node (top of the stack)
    pub fn focused_node(&self) -> Option<&TreeNode> {
        self.view
            .focus_stack
            .last()
            .and_then(|&index| self.snapshot.nodes.get(index))
    }

    pub fn update_scroll(&mut self, viewport_height: usize, cursor_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.view.cursor < self.view.scroll_offset {
            self.view.scroll_offset = self.view.cursor;
        } else if self.view.cursor + cursor_height > self.view.scroll_offset + viewport_height {
            self.view.scroll_offset =
                (self.view.cursor + cursor_height).saturating_sub(viewport_height);
        }
    }

    pub fn page_up(&mut self, amount: usize) {
        self.view.cursor = self.view.cursor.saturating_sub(amount);
        self.sync_neighborhood_to_cursor();
    }

    pub fn page_down(&mut self, amount: usize) {
        let max = self.visible_count().saturating_sub(1);
        self.view.cursor = (self.view.cursor + amount).min(max);
        self.sync_neighborhood_to_cursor();
    }

    pub fn toggle_expanded(&mut self) {
        if self.view.expanded_entry == Some(self.view.cursor) {
            self.view.expanded_entry = None;
        } else {
            self.view.expanded_entry = Some(self.view.cursor);
        }
    }

    pub fn is_expanded(&self, visible_idx: usize) -> bool {
        self.view.expanded_entry == Some(visible_idx)
    }

    /// Build a map of bookmark names to their visible entry indices
    pub fn bookmark_to_visible_index(&self) -> HashMap<String, usize> {
        let mut map = HashMap::default();
        for (visible_idx, entry) in self.projection.visible_entries.iter().enumerate() {
            let node = &self.snapshot.nodes[entry.node_index];
            for bookmark in &node.bookmarks {
                map.insert(bookmark.name.clone(), visible_idx);
            }
        }
        map
    }

    pub fn toggle_selected(&mut self, idx: usize) {
        if self.view.selected.contains(&idx) {
            self.view.selected.remove(&idx);
        } else {
            self.view.selected.insert(idx);
        }
    }

    pub fn select_range(&mut self, from: usize, to: usize) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };
        for index in start..=end {
            self.view.selected.insert(index);
        }
    }

    pub fn clear_selection(&mut self) {
        self.view.selected.clear();
        self.view.selection_anchor = None;
    }

    fn restore_cursor_to_change_id(&mut self, change_id: &str) {
        if let Some(index) = self
            .projection
            .visible_entries
            .iter()
            .position(|entry| self.snapshot.nodes[entry.node_index].change_id == change_id)
        {
            self.view.cursor = index;
        }
    }

    fn sync_neighborhood_to_cursor(&mut self) {
        if !self.is_neighborhood_following_cursor() {
            return;
        }

        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return;
        };

        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
    }
}

fn focused_root_index(view: &TreeViewState) -> Option<usize> {
    if matches!(&view.view_mode, ViewMode::Neighborhood(..)) {
        None
    } else {
        view.focus_stack.last().copied()
    }
}

fn neighborhood_state(view_mode: &ViewMode) -> Option<&NeighborhoodState> {
    match view_mode {
        ViewMode::Neighborhood(state) => Some(state),
        ViewMode::Tree => None,
    }
}

fn resolve_neighborhood_anchor_index(
    snapshot: &TreeSnapshot,
    view: &TreeViewState,
    current_entry_node_index: Option<usize>,
) -> Option<usize> {
    let state = neighborhood_state(&view.view_mode)?;
    match &state.anchor {
        NeighborhoodAnchor::FollowCursor => current_entry_node_index
            .or_else(|| snapshot.nodes.iter().position(|node| node.is_working_copy)),
        NeighborhoodAnchor::Fixed(change_id) => snapshot
            .nodes
            .iter()
            .position(|node| node.change_id == *change_id)
            .or(current_entry_node_index),
    }
}
