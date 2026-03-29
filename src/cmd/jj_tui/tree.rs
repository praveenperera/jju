mod topology;

use crate::jj_lib_helpers::JjRepo;
use ahash::{HashMap, HashSet};
use eyre::Result;
use jj_lib::object_id::ObjectId;
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
        self.bookmarks.iter().map(|b| b.name.clone()).collect()
    }

    /// Check if any bookmark has the given name
    pub fn has_bookmark(&self, name: &str) -> bool {
        self.bookmarks.iter().any(|b| b.name == name)
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

/// Check if a subtree rooted at `root` contains `target` via DFS through children_map
fn subtree_contains(root: &str, target: &str, children_map: &HashMap<String, Vec<String>>) -> bool {
    if root == target {
        return true;
    }
    let mut stack = vec![root.to_string()];
    let mut visited = HashSet::default();
    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        if let Some(children) = children_map.get(&node) {
            for child in children {
                if child == target {
                    return true;
                }
                stack.push(child.clone());
            }
        }
    }
    false
}

impl TreeState {
    pub fn load(jj_repo: &JjRepo) -> Result<Self> {
        Self::load_with_base(jj_repo, "trunk()")
    }

    pub fn load_with_base(jj_repo: &JjRepo, base: &str) -> Result<Self> {
        let working_copy = jj_repo.working_copy_commit()?;
        let working_copy_id = jj_repo.shortest_change_id(&working_copy, 4)?;

        // show all mutable commits (same as jj log default), rooted at base
        let revset = format!("{base} | ancestors(immutable_heads().., 2) | @::");
        let commits = jj_repo.eval_revset(&revset)?;

        let mut commit_map: HashMap<String, TreeNode> = HashMap::default();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::default();

        for commit in &commits {
            let (change_id, unique_prefix_len) = jj_repo.change_id_with_prefix_len(commit, 4)?;
            let (commit_id, unique_commit_prefix_len) =
                jj_repo.commit_id_with_prefix_len(commit, 7)?;
            let bookmarks: Vec<BookmarkInfo> = jj_repo
                .bookmarks_with_state(commit)
                .into_iter()
                .map(|(name, is_diverged)| BookmarkInfo { name, is_diverged })
                .collect();
            let description = JjRepo::description_first_line(commit);
            let full_description = commit.description().to_string();

            let parents = jj_repo.parent_commits(commit)?;
            let parent_ids: Vec<String> = parents
                .iter()
                .filter_map(|p| jj_repo.shortest_change_id(p, 4).ok())
                .collect();

            let is_working_copy = change_id == working_copy_id;
            let has_conflicts = JjRepo::has_conflict(commit);

            let author_name = JjRepo::author_name(commit);
            let author_email = JjRepo::author_email(commit);
            let timestamp = JjRepo::author_timestamp_relative(commit);

            // check for divergence
            let is_divergent = jj_repo.is_commit_divergent(commit);
            let divergent_versions = if is_divergent {
                jj_repo
                    .get_divergent_commits(commit)
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                    .map(|(idx, c)| {
                        let c_id = c.id().hex();
                        // heuristic: index 0 is newest (local), or has working copy
                        let is_local = idx == 0 || c_id == commit.id().hex() && is_working_copy;
                        DivergentVersion {
                            commit_id: c_id,
                            is_local,
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let node = TreeNode {
                change_id: change_id.clone(),
                unique_prefix_len,
                commit_id,
                unique_commit_prefix_len,
                description,
                full_description,
                bookmarks,
                is_working_copy,
                has_conflicts,
                is_divergent,
                divergent_versions,
                parent_ids: parent_ids.clone(),
                depth: 0,
                author_name,
                author_email,
                timestamp,
            };

            commit_map.insert(change_id.clone(), node);

            for parent_id in parent_ids {
                children_map
                    .entry(parent_id)
                    .or_default()
                    .push(change_id.clone());
            }
        }

        if commit_map.is_empty() {
            return Ok(Self {
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
            });
        }

        // get base change_id for root detection
        let base_id = jj_repo
            .eval_revset_single(base)
            .ok()
            .and_then(|c| jj_repo.shortest_change_id(&c, 4).ok());

        // find roots (commits whose parents aren't in our set, OR the base itself)
        let revs_in_set: HashSet<&str> = commit_map.keys().map(|s| s.as_str()).collect();
        let mut roots: Vec<String> = commit_map
            .values()
            .filter(|c| {
                // always include base as root
                if let Some(ref bid) = base_id
                    && c.change_id == *bid
                {
                    return true;
                }
                c.parent_ids
                    .iter()
                    .all(|p| !revs_in_set.contains(p.as_str()))
            })
            .map(|c| c.change_id.clone())
            .collect();
        roots.sort();

        // order roots: working copy tree first (if different from base), then base, then others
        let wc_root = roots
            .iter()
            .find(|r| subtree_contains(r, &working_copy_id, &children_map))
            .cloned();
        let base_root = base_id
            .as_ref()
            .and_then(|bid| roots.iter().find(|r| *r == bid).cloned());

        let mut ordered_roots = Vec::with_capacity(roots.len());
        if let Some(ref wc) = wc_root {
            ordered_roots.push(wc.clone());
        }
        if let Some(ref br) = base_root
            && Some(br) != wc_root.as_ref()
        {
            ordered_roots.push(br.clone());
        }
        for r in &roots {
            if !ordered_roots.contains(r) {
                ordered_roots.push(r.clone());
            }
        }
        let roots = ordered_roots;

        let mut nodes = Vec::new();
        let mut visited = HashSet::default();

        fn traverse(
            change_id: &str,
            commit_map: &HashMap<String, TreeNode>,
            children_map: &HashMap<String, Vec<String>>,
            nodes: &mut Vec<TreeNode>,
            visited: &mut HashSet<String>,
            depth: usize,
        ) {
            if visited.contains(change_id) {
                return;
            }
            visited.insert(change_id.to_string());

            if let Some(node) = commit_map.get(change_id) {
                let mut node = node.clone();
                node.depth = depth;
                nodes.push(node);

                if let Some(children) = children_map.get(change_id) {
                    let mut sorted_children = children.clone();
                    sorted_children.sort();
                    for child in sorted_children {
                        traverse(&child, commit_map, children_map, nodes, visited, depth + 1);
                    }
                }
            }
        }

        for root in &roots {
            traverse(
                root,
                &commit_map,
                &children_map,
                &mut nodes,
                &mut visited,
                0,
            );
        }

        let topology = TreeTopology::from_nodes(&nodes);
        let visible_entries = Self::compute_visible_entries(&nodes, &topology, true, None);

        Ok(Self {
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
        })
    }

    fn compute_visible_entries(
        nodes: &[TreeNode],
        topology: &TreeTopology,
        full_mode: bool,
        focused_root: Option<usize>,
    ) -> Vec<VisibleEntry> {
        let (filtered_nodes, base_depth): (Vec<(usize, &TreeNode)>, usize) =
            if let Some(root_idx) = focused_root {
                if root_idx >= nodes.len() {
                    return Vec::new();
                }
                let root_depth = nodes[root_idx].depth;
                (
                    topology
                        .subtree_nodes_in_order(root_idx)
                        .into_iter()
                        .map(|node_index| (node_index, &nodes[node_index]))
                        .collect(),
                    root_depth,
                )
            } else {
                (nodes.iter().enumerate().collect(), 0)
            };

        if full_mode {
            // in full mode, use structural depth (minus base_depth for focused)
            let mut is_first = true;
            filtered_nodes
                .iter()
                .map(|(i, n)| {
                    let depth = n.depth.saturating_sub(base_depth);
                    // separator before each new root tree (depth 0) except the first
                    let has_separator_before = depth == 0 && !is_first;
                    if depth == 0 {
                        is_first = false;
                    }
                    VisibleEntry {
                        node_index: *i,
                        visual_depth: depth,
                        has_separator_before,
                    }
                })
                .collect()
        } else {
            let mut entries = Vec::new();
            let mut seen_root = false;
            let visible_nodes: Vec<usize> = filtered_nodes
                .iter()
                .filter_map(|(node_index, node)| node.is_visible(full_mode).then_some(*node_index))
                .collect();
            let visible_set: HashSet<usize> = visible_nodes.iter().copied().collect();
            let mut visual_depths: HashMap<usize, usize> = HashMap::default();

            for (node_index, node) in filtered_nodes {
                if !node.is_visible(full_mode) {
                    continue;
                }

                let mut current_parent = topology.parent_of(node_index);
                let mut visual_depth = 0;
                while let Some(parent_index) = current_parent {
                    if visible_set.contains(&parent_index) {
                        visual_depth = visual_depths
                            .get(&parent_index)
                            .copied()
                            .unwrap_or(0)
                            .saturating_add(1);
                        break;
                    }
                    current_parent = topology.parent_of(parent_index);
                }
                visual_depths.insert(node_index, visual_depth);

                let has_separator_before = visual_depth == 0 && seen_root;
                if visual_depth == 0 {
                    seen_root = true;
                }

                entries.push(VisibleEntry {
                    node_index,
                    visual_depth,
                    has_separator_before,
                });
            }
            entries
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
        self.current_entry().map(|e| &self.nodes[e.node_index])
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
        for (i, entry) in self.visible_entries.iter().enumerate() {
            if self.nodes[entry.node_index].is_working_copy {
                self.cursor = i;
                return;
            }
        }
    }

    pub fn toggle_full_mode(&mut self) {
        self.full_mode = !self.full_mode;
        self.recompute_visible_entries();
    }

    fn recompute_visible_entries(&mut self) {
        let focused_root = self.focus_stack.last().copied();
        self.visible_entries = Self::compute_visible_entries(
            &self.nodes,
            &self.topology,
            self.full_mode,
            focused_root,
        );

        if self.cursor >= self.visible_count() {
            self.cursor = self.visible_count().saturating_sub(1);
        }

        // clear expanded entry when view changes to avoid stale index
        self.expanded_entry = None;
    }

    /// Toggle focus on the current node
    pub fn toggle_focus(&mut self) {
        let Some(entry) = self.current_entry() else {
            return;
        };
        let current_node_idx = entry.node_index;

        // if on the current focus root at cursor 0, zoom out one level
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
            .and_then(|idx| self.nodes.get(idx).map(|n| n.change_id.clone()));

        self.recompute_visible_entries();

        // restore cursor to the previously focused node
        if let Some(change_id) = popped_change_id
            && let Some(idx) = self
                .visible_entries
                .iter()
                .position(|e| self.nodes[e.node_index].change_id == change_id)
        {
            self.cursor = idx;
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
        self.focus_stack.last().and_then(|&idx| self.nodes.get(idx))
    }

    pub fn update_scroll(&mut self, viewport_height: usize, cursor_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor + cursor_height > self.scroll_offset + viewport_height {
            // scroll so cursor row + its expanded content fits
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
        for i in start..=end {
            self.selected.insert(i);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.selection_anchor = None;
    }
}
