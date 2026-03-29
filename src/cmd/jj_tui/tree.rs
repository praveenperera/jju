mod load;
mod topology;
mod visible;

use crate::jj_lib_helpers::JjRepo;
use ahash::{HashMap, HashSet};
use eyre::Result;
pub use topology::TreeTopology;

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
    pub unique_commit_prefix_len: usize,
    pub description: String,
    pub full_description: String,
    pub bookmarks: Vec<BookmarkInfo>,
    pub is_working_copy: bool,
    pub has_conflicts: bool,
    pub is_divergent: bool,
    pub divergent_versions: Vec<DivergentVersion>, // all versions if divergent
    pub parent_ids: Vec<String>,
    pub depth: usize,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: String,
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
}

pub struct VisibleEntry {
    pub node_index: usize,
    pub visual_depth: usize,
    pub has_separator_before: bool,
}

pub struct TreeState {
    pub nodes: Vec<TreeNode>,
    pub topology: TreeTopology,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub full_mode: bool,
    pub expanded_entry: Option<usize>,
    pub visible_entries: Vec<VisibleEntry>,
    pub selected: HashSet<usize>,
    pub selection_anchor: Option<usize>,
    pub focus_stack: Vec<usize>, // stack of node_indices for nested zoom
}

impl TreeState {
    pub fn load(jj_repo: &JjRepo) -> Result<Self> {
        load::load_tree_state(jj_repo, "trunk()")
    }

    pub fn load_with_base(jj_repo: &JjRepo, base: &str) -> Result<Self> {
        load::load_tree_state(jj_repo, base)
    }

    fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            topology: TreeTopology::default(),
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            expanded_entry: None,
            visible_entries: Vec::new(),
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }

    fn from_nodes(nodes: Vec<TreeNode>) -> Self {
        let topology = TreeTopology::from_nodes(&nodes);
        let visible_entries = visible::compute_visible_entries(&nodes, &topology, true, None);

        Self {
            nodes,
            topology,
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            expanded_entry: None,
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }

    pub fn visible_nodes(&self) -> impl Iterator<Item = &VisibleEntry> {
        self.visible_entries.iter()
    }

    pub fn get_node(&self, entry: &VisibleEntry) -> &TreeNode {
        &self.nodes[entry.node_index]
    }

    pub fn visible_count(&self) -> usize {
        self.visible_entries.len()
    }

    pub fn current_entry(&self) -> Option<&VisibleEntry> {
        self.visible_entries.get(self.cursor)
    }

    pub fn current_node(&self) -> Option<&TreeNode> {
        self.current_entry()
            .map(|entry| &self.nodes[entry.node_index])
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor + 1 < self.visible_count() {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_top(&mut self) {
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    pub fn move_cursor_bottom(&mut self) {
        let count = self.visible_count();
        if count > 0 {
            self.cursor = count - 1;
        }
    }

    pub fn jump_to_working_copy(&mut self) {
        for (index, entry) in self.visible_entries.iter().enumerate() {
            if self.nodes[entry.node_index].is_working_copy {
                self.cursor = index;
                return;
            }
        }
    }

    pub fn toggle_full_mode(&mut self) {
        self.full_mode = !self.full_mode;
        self.recompute_visible_entries();
    }

    fn recompute_visible_entries(&mut self) {
        self.visible_entries = visible::compute_visible_entries(
            &self.nodes,
            &self.topology,
            self.full_mode,
            self.focus_stack.last().copied(),
        );

        if self.cursor >= self.visible_count() {
            self.cursor = self.visible_count().saturating_sub(1);
        }

        self.expanded_entry = None;
    }

    /// Toggle focus on the current node
    pub fn toggle_focus(&mut self) {
        let Some(entry) = self.current_entry() else {
            return;
        };
        let current_node_idx = entry.node_index;

        if self.focus_stack.last() == Some(&current_node_idx) && self.cursor == 0 {
            self.unfocus();
            return;
        }

        self.focus_on(current_node_idx);
    }

    /// Focus on a specific node (zoom in), pushing to the focus stack
    pub fn focus_on(&mut self, node_index: usize) {
        self.focus_stack.push(node_index);
        self.recompute_visible_entries();
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    /// Unfocus one level (zoom out), popping from the focus stack
    pub fn unfocus(&mut self) {
        let popped_change_id = self
            .focus_stack
            .pop()
            .and_then(|index| self.nodes.get(index).map(|node| node.change_id.clone()));

        self.recompute_visible_entries();

        if let Some(change_id) = popped_change_id
            && let Some(index) = self
                .visible_entries
                .iter()
                .position(|entry| self.nodes[entry.node_index].change_id == change_id)
        {
            self.cursor = index;
        }
    }

    /// Returns true if the tree is currently focused (zoomed)
    pub fn is_focused(&self) -> bool {
        !self.focus_stack.is_empty()
    }

    /// Returns the current focus depth (number of zoom levels)
    pub fn focus_depth(&self) -> usize {
        self.focus_stack.len()
    }

    /// Get the currently focused node (top of the stack)
    pub fn focused_node(&self) -> Option<&TreeNode> {
        self.focus_stack
            .last()
            .and_then(|&index| self.nodes.get(index))
    }

    pub fn update_scroll(&mut self, viewport_height: usize, cursor_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor + cursor_height > self.scroll_offset + viewport_height {
            self.scroll_offset = (self.cursor + cursor_height).saturating_sub(viewport_height);
        }
    }

    pub fn page_up(&mut self, amount: usize) {
        self.cursor = self.cursor.saturating_sub(amount);
    }

    pub fn page_down(&mut self, amount: usize) {
        let max = self.visible_count().saturating_sub(1);
        self.cursor = (self.cursor + amount).min(max);
    }

    pub fn toggle_expanded(&mut self) {
        if self.expanded_entry == Some(self.cursor) {
            self.expanded_entry = None;
        } else {
            self.expanded_entry = Some(self.cursor);
        }
    }

    pub fn is_expanded(&self, visible_idx: usize) -> bool {
        self.expanded_entry == Some(visible_idx)
    }

    /// Build a map of bookmark names to their visible entry indices
    pub fn bookmark_to_visible_index(&self) -> HashMap<String, usize> {
        let mut map = HashMap::default();
        for (visible_idx, entry) in self.visible_entries.iter().enumerate() {
            let node = &self.nodes[entry.node_index];
            for bookmark in &node.bookmarks {
                map.insert(bookmark.name.clone(), visible_idx);
            }
        }
        map
    }

    pub fn toggle_selected(&mut self, idx: usize) {
        if self.selected.contains(&idx) {
            self.selected.remove(&idx);
        } else {
            self.selected.insert(idx);
        }
    }

    pub fn select_range(&mut self, from: usize, to: usize) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };
        for index in start..=end {
            self.selected.insert(index);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.selection_anchor = None;
    }
}
