use super::tree::{TreeNode, TreeState};
use ahash::{HashMap, HashSet};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Unique identifier for a node in the preview tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Type of rebase operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewRebaseType {
    Single,          // -r: just this revision
    WithDescendants, // -s: revision + all descendants
}

/// Role of a node in the preview
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Normal,      // unchanged
    Source,      // the commit being rebased (source of operation)
    Moving,      // descendant of source (moving with it in -s mode)
    Destination, // the target where source will be placed
    #[allow(dead_code)]
    Shifted, // will shift position due to rebase
}

/// Mode for rendering markers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerMode {
    #[allow(dead_code)]
    None,
    Rebase { allow_branches: bool },
}

/// A slot in the preview display
#[derive(Debug, Clone)]
pub struct DisplaySlot {
    pub node_id: NodeId,
    pub visual_depth: usize,
    pub role: NodeRole,
}

/// Preview of tree state after operation
#[allow(dead_code)]
pub struct Preview {
    pub slots: Vec<DisplaySlot>,
    pub source_id: Option<NodeId>,
    pub dest_id: Option<NodeId>,
}

/// Relations between nodes (parent-child)
struct TreeRelations {
    parent: HashMap<NodeId, NodeId>,
    children: HashMap<NodeId, Vec<NodeId>>,
}

impl TreeRelations {
    fn from_tree(tree: &TreeState) -> Self {
        let mut parent = HashMap::default();
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::default();

        // build parent-child relations from structural depth
        let mut depth_stack: Vec<(usize, NodeId)> = Vec::new();

        for (idx, entry) in tree.visible_entries.iter().enumerate() {
            let node = &tree.nodes[entry.node_index];
            let node_id = NodeId(idx);

            // pop stack until we find a potential parent (smaller depth)
            while let Some(&(parent_depth, _)) = depth_stack.last() {
                if parent_depth < node.depth {
                    break;
                }
                depth_stack.pop();
            }

            // the top of stack is our parent
            if let Some(&(_, parent_id)) = depth_stack.last() {
                parent.insert(node_id, parent_id);
                children.entry(parent_id).or_default().push(node_id);
            }

            depth_stack.push((node.depth, node_id));
        }

        Self { parent, children }
    }

    fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        self.parent.get(&node).copied()
    }

    fn children_of(&self, node: NodeId) -> &[NodeId] {
        self.children.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all descendants of a node (recursive)
    fn descendants(&self, node: NodeId) -> HashSet<NodeId> {
        let mut result = HashSet::default();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            for &child in self.children_of(current) {
                if result.insert(child) {
                    stack.push(child);
                }
            }
        }

        result
    }

    /// Set a new parent for a node
    #[allow(dead_code)]
    fn set_parent(&mut self, child: NodeId, new_parent: NodeId) {
        // remove from old parent's children
        if let Some(old_parent) = self.parent.get(&child).copied() {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&id| id != child);
            }
        }

        // set new parent
        self.parent.insert(child, new_parent);
        self.children.entry(new_parent).or_default().push(child);
    }

    /// Remove a node from its parent (for reparenting source's children)
    fn remove_from_parent(&mut self, child: NodeId) {
        if let Some(old_parent) = self.parent.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&old_parent) {
                siblings.retain(|&id| id != child);
            }
        }
    }

    /// Add a node as child of another
    fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.parent.insert(child, parent);
        self.children.entry(parent).or_default().push(child);
    }
}

pub struct PreviewBuilder<'a> {
    tree: &'a TreeState,
    relations: TreeRelations,
}

impl<'a> PreviewBuilder<'a> {
    pub fn new(tree: &'a TreeState) -> Self {
        let relations = TreeRelations::from_tree(tree);
        Self { tree, relations }
    }

    /// Find the last node in a chain (the leaf/tip of the moving subtree)
    /// Follows children until finding a node with no children in the moving set
    fn find_last_descendant(&self, start: NodeId, moving_ids: &HashSet<NodeId>) -> NodeId {
        let mut current = start;
        loop {
            // Find children that are in the moving set
            let moving_children: Vec<NodeId> = self
                .relations
                .children_of(current)
                .iter()
                .copied()
                .filter(|c| moving_ids.contains(c))
                .collect();

            if moving_children.is_empty() {
                // No more moving children, this is the last node
                return current;
            }
            // Follow the first moving child (in a linear chain, there should be only one)
            current = moving_children[0];
        }
    }

    /// Build a rebase preview using data-first approach
    pub fn rebase_preview(
        mut self,
        source: NodeId,
        dest: NodeId,
        rebase_type: PreviewRebaseType,
        allow_branches: bool,
    ) -> Preview {
        // no-op if source == dest
        if source == dest {
            return self.identity_preview(source, dest);
        }

        // collect moving nodes (source + descendants in -s mode)
        let mut moving_ids: HashSet<NodeId> = HashSet::default();
        moving_ids.insert(source);

        if rebase_type == PreviewRebaseType::WithDescendants {
            let descendants = self.relations.descendants(source);
            moving_ids.extend(descendants);
        }

        // get source's original parent and children
        let source_parent = self.relations.parent_of(source);
        let source_children: Vec<NodeId> = self.relations.children_of(source).to_vec();

        // STEP 1: Reparent source's children to source's old parent (for -r mode)
        if rebase_type == PreviewRebaseType::Single {
            for &child in &source_children {
                if !moving_ids.contains(&child) {
                    self.relations.remove_from_parent(child);
                    if let Some(parent) = source_parent {
                        self.relations.add_child(parent, child);
                    }
                }
            }
        }

        // STEP 2: Handle dest's children based on fork/inline mode
        let dest_children: Vec<NodeId> = self.relations.children_of(dest).to_vec();

        if !allow_branches && !dest_children.is_empty() {
            // INLINE mode: dest's children go to the END of the moving chain
            // Find the last node in the moving chain (leaf of the moving subtree)
            let last_moving = self.find_last_descendant(source, &moving_ids);

            for &child in &dest_children {
                if !moving_ids.contains(&child) {
                    self.relations.remove_from_parent(child);
                    self.relations.add_child(last_moving, child);
                }
            }
        }

        // STEP 3: Set source's new parent to dest
        self.relations.remove_from_parent(source);
        self.relations.add_child(dest, source);

        // STEP 4: Build slots via DFS traversal (depth emerges from traversal)
        let slots = self.build_slots_dfs(&moving_ids, source, dest);

        Preview {
            slots,
            source_id: Some(source),
            dest_id: Some(dest),
        }
    }

    /// Build preview where nothing changes
    fn identity_preview(&self, source: NodeId, dest: NodeId) -> Preview {
        let slots: Vec<DisplaySlot> = self
            .tree
            .visible_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let node_id = NodeId(idx);
                let role = if node_id == source {
                    NodeRole::Source
                } else if node_id == dest {
                    NodeRole::Destination
                } else {
                    NodeRole::Normal
                };
                DisplaySlot {
                    node_id,
                    visual_depth: entry.visual_depth,
                    role,
                }
            })
            .collect();

        Preview {
            slots,
            source_id: Some(source),
            dest_id: Some(dest),
        }
    }

    /// Build slots using DFS traversal - depth emerges from traversal level
    fn build_slots_dfs(
        &self,
        moving_ids: &HashSet<NodeId>,
        source: NodeId,
        dest: NodeId,
    ) -> Vec<DisplaySlot> {
        let mut slots = Vec::new();
        let mut visited = HashSet::default();

        // find roots (nodes with no parent in our tree)
        let roots: Vec<NodeId> = (0..self.tree.visible_entries.len())
            .map(NodeId)
            .filter(|&id| self.relations.parent_of(id).is_none())
            .collect();

        for root in roots {
            self.dfs_traverse(root, 0, moving_ids, source, dest, &mut slots, &mut visited);
        }

        slots
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs_traverse(
        &self,
        node_id: NodeId,
        depth: usize,
        moving_ids: &HashSet<NodeId>,
        source: NodeId,
        dest: NodeId,
        slots: &mut Vec<DisplaySlot>,
        visited: &mut HashSet<NodeId>,
    ) {
        if !visited.insert(node_id) {
            return;
        }

        let role = if node_id == source {
            NodeRole::Source
        } else if node_id == dest {
            NodeRole::Destination
        } else if moving_ids.contains(&node_id) {
            NodeRole::Moving
        } else {
            NodeRole::Normal
        };

        slots.push(DisplaySlot {
            node_id,
            visual_depth: depth,
            role,
        });

        // recurse into children
        for &child in self.relations.children_of(node_id) {
            self.dfs_traverse(child, depth + 1, moving_ids, source, dest, slots, visited);
        }
    }
}

/// Get a TreeNode by NodeId
pub fn get_node(tree: &TreeState, node_id: NodeId) -> &TreeNode {
    let entry = &tree.visible_entries[node_id.0];
    &tree.nodes[entry.node_index]
}

/// Render a single tree line with preview markers
pub fn render_tree_line(
    node: &TreeNode,
    visual_depth: usize,
    is_cursor: bool,
    role: NodeRole,
    marker_mode: MarkerMode,
) -> Line<'static> {
    let indent = "  ".repeat(visual_depth);
    let connector = if visual_depth > 0 { "├── " } else { "" };
    let at_marker = if node.is_working_copy { "@ " } else { "" };

    let (prefix, suffix) = node
        .change_id
        .split_at(node.unique_prefix_len.min(node.change_id.len()));

    let mut spans = Vec::new();

    // change_id color based on role
    let prefix_color = match role {
        NodeRole::Source | NodeRole::Moving => Color::Yellow,
        _ => Color::Magenta,
    };

    let dim_color = Color::Reset;

    spans.extend([
        Span::raw(format!("{indent}{connector}{at_marker}(")),
        Span::styled(prefix.to_string(), Style::default().fg(prefix_color)),
        Span::styled(
            suffix.to_string(),
            Style::default().add_modifier(Modifier::DIM),
        ),
        Span::raw(")"),
    ]);

    if !node.bookmarks.is_empty() {
        let bookmark_str = node
            .bookmarks
            .iter()
            .map(|b| {
                if b.is_diverged {
                    format!("{}*", b.name)
                } else {
                    b.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        spans.push(Span::raw(" "));
        spans.push(Span::styled(bookmark_str, Style::default().fg(Color::Cyan)));
    }

    let desc = if node.description.is_empty() {
        if node.is_working_copy {
            "(working copy)".to_string()
        } else {
            "(no description)".to_string()
        }
    } else {
        node.description.clone()
    };
    spans.push(Span::styled(format!("  {desc}"), Style::default().fg(dim_color)));

    // add markers based on role and mode
    match (role, marker_mode) {
        (NodeRole::Source, MarkerMode::Rebase { .. }) => {
            spans.push(Span::styled("  ← src", Style::default().fg(Color::Yellow)));
        }
        (NodeRole::Destination, MarkerMode::Rebase { allow_branches }) => {
            let mode_hint = if allow_branches { "fork" } else { "inline" };
            spans.push(Span::styled(
                format!("  ← dest ({mode_hint})"),
                Style::default().fg(Color::Cyan),
            ));
        }
        (NodeRole::Moving, MarkerMode::Rebase { .. }) => {
            spans.push(Span::styled("  ↳", Style::default().fg(Color::Yellow)));
        }
        _ => {}
    }

    let mut line = Line::from(spans);

    // apply styling based on state
    if is_cursor {
        line = line.style(
            Style::default()
                .bg(Color::Rgb(40, 40, 60))
                .add_modifier(Modifier::BOLD),
        );
    } else if matches!(role, NodeRole::Source | NodeRole::Moving) {
        line = line.style(Style::default().bg(Color::Rgb(50, 50, 30)));
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::tree::{TreeNode, TreeState, VisibleEntry};
    use ahash::{HashMap, HashSet};

    fn make_node(change_id: &str, depth: usize) -> TreeNode {
        TreeNode {
            change_id: change_id.to_string(),
            unique_prefix_len: 4,
            description: String::new(),
            full_description: String::new(),
            bookmarks: vec![],
            is_working_copy: false,
            parent_ids: vec![],
            depth,
            author_name: String::new(),
            author_email: String::new(),
            timestamp: String::new(),
        }
    }

    fn make_tree(nodes: Vec<TreeNode>, full_mode: bool) -> TreeState {
        let visible_entries: Vec<VisibleEntry> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| VisibleEntry {
                node_index: i,
                visual_depth: n.depth,
            })
            .collect();

        TreeState {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            full_mode,
            expanded_entry: None,
            children_map: HashMap::default(),
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }

    #[test]
    fn test_from_tree_linear() {
        // Linear tree: A -> B -> C -> D
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
                make_node("dddd", 3),
            ],
            true,
        );

        let relations = TreeRelations::from_tree(&tree);

        // Check parent relationships
        assert_eq!(relations.parent_of(NodeId(0)), None); // A is root
        assert_eq!(relations.parent_of(NodeId(1)), Some(NodeId(0))); // B's parent is A
        assert_eq!(relations.parent_of(NodeId(2)), Some(NodeId(1))); // C's parent is B
        assert_eq!(relations.parent_of(NodeId(3)), Some(NodeId(2))); // D's parent is C

        // Check children
        assert_eq!(relations.children_of(NodeId(0)), &[NodeId(1)]);
        assert_eq!(relations.children_of(NodeId(1)), &[NodeId(2)]);
        assert_eq!(relations.children_of(NodeId(2)), &[NodeId(3)]);
        assert_eq!(relations.children_of(NodeId(3)), &[] as &[NodeId]);
    }

    #[test]
    fn test_from_tree_forked() {
        // Forked tree:
        // A (depth 0)
        //   B (depth 1)
        //     C (depth 2)
        //   D (depth 1) <- sibling of B
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
                make_node("dddd", 1), // same depth as B
            ],
            true,
        );

        let relations = TreeRelations::from_tree(&tree);

        // Check parent relationships
        assert_eq!(relations.parent_of(NodeId(0)), None); // A is root
        assert_eq!(relations.parent_of(NodeId(1)), Some(NodeId(0))); // B's parent is A
        assert_eq!(relations.parent_of(NodeId(2)), Some(NodeId(1))); // C's parent is B
        assert_eq!(relations.parent_of(NodeId(3)), Some(NodeId(0))); // D's parent is A

        // Check children
        let children_of_a = relations.children_of(NodeId(0));
        assert!(children_of_a.contains(&NodeId(1))); // B
        assert!(children_of_a.contains(&NodeId(3))); // D
        assert_eq!(children_of_a.len(), 2);
    }

    #[test]
    fn test_descendants() {
        // Linear tree: A -> B -> C
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
            ],
            true,
        );

        let relations = TreeRelations::from_tree(&tree);

        let desc_a = relations.descendants(NodeId(0));
        assert!(desc_a.contains(&NodeId(1)));
        assert!(desc_a.contains(&NodeId(2)));
        assert_eq!(desc_a.len(), 2);

        let desc_b = relations.descendants(NodeId(1));
        assert!(desc_b.contains(&NodeId(2)));
        assert_eq!(desc_b.len(), 1);

        let desc_c = relations.descendants(NodeId(2));
        assert!(desc_c.is_empty());
    }

    #[test]
    fn test_rebase_with_descendants() {
        // Tree:
        // A (depth 0)
        //   B (depth 1)
        //     C (depth 2)
        //     D (depth 2)
        // Rebase B with descendants onto A (should be no-op essentially, but test the slots)
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
                make_node("dddd", 2),
            ],
            true,
        );

        let preview = PreviewBuilder::new(&tree).rebase_preview(
            NodeId(1), // source = B
            NodeId(0), // dest = A (B's current parent)
            PreviewRebaseType::WithDescendants,
            true, // allow_branches (fork mode)
        );

        // In fork mode with B already child of A, should get:
        // A at depth 0
        //   B at depth 1 (source)
        //     C at depth 2 (moving)
        //     D at depth 2 (moving)

        assert_eq!(preview.slots.len(), 4);

        // Find each node's slot
        let slot_a = preview.slots.iter().find(|s| s.node_id == NodeId(0)).unwrap();
        let slot_b = preview.slots.iter().find(|s| s.node_id == NodeId(1)).unwrap();
        let slot_c = preview.slots.iter().find(|s| s.node_id == NodeId(2)).unwrap();
        let slot_d = preview.slots.iter().find(|s| s.node_id == NodeId(3)).unwrap();

        assert_eq!(slot_a.visual_depth, 0);
        assert_eq!(slot_b.visual_depth, 1);
        assert_eq!(slot_c.visual_depth, 2);
        assert_eq!(slot_d.visual_depth, 2);

        assert_eq!(slot_b.role, NodeRole::Source);
        assert_eq!(slot_c.role, NodeRole::Moving);
        assert_eq!(slot_d.role, NodeRole::Moving);
    }

    #[test]
    fn test_non_full_mode_with_hidden_nodes() {
        // Simulate non-full mode where some nodes are hidden
        // Actual tree (with hidden nodes):
        // A (depth 0) visible
        //   B (depth 1) HIDDEN
        //     C (depth 2) visible
        //       D (depth 3) visible
        //
        // visible_entries would be: A, C, D with visual_depths 0, 1, 2
        // But node.depth would be: 0, 2, 3

        // Create nodes with their structural depths
        let nodes = vec![
            make_node("aaaa", 0), // A
            make_node("cccc", 2), // C (B is hidden, so structural depth is still 2)
            make_node("dddd", 3), // D
        ];

        // But visible_entries have visual_depth based on visible ancestors
        let visible_entries = vec![
            VisibleEntry {
                node_index: 0,
                visual_depth: 0,
            },
            VisibleEntry {
                node_index: 1,
                visual_depth: 1, // visual depth is 1 (child of visible A)
            },
            VisibleEntry {
                node_index: 2,
                visual_depth: 2, // visual depth is 2
            },
        ];

        let tree = TreeState {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            full_mode: false,
            expanded_entry: None,
            children_map: HashMap::default(),
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        };

        let relations = TreeRelations::from_tree(&tree);

        // from_tree uses node.depth, not visual_depth
        // So it sees depths: 0, 2, 3
        // This should give: A (0), C (2) -> parent A, D (3) -> parent C ✓

        assert_eq!(relations.parent_of(NodeId(0)), None); // A is root
        assert_eq!(relations.parent_of(NodeId(1)), Some(NodeId(0))); // C's parent is A
        assert_eq!(relations.parent_of(NodeId(2)), Some(NodeId(1))); // D's parent is C
    }

    #[test]
    fn test_rebase_subtree_to_different_parent() {
        // Tree:
        // A (depth 0)
        //   B (depth 1)
        //     C (depth 2)  <- source
        //       D (depth 3) <- descendant
        // Rebase C with descendants onto A
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
                make_node("dddd", 3),
            ],
            true,
        );

        let preview = PreviewBuilder::new(&tree).rebase_preview(
            NodeId(2), // source = C
            NodeId(0), // dest = A
            PreviewRebaseType::WithDescendants,
            true, // fork mode
        );

        // Expected result:
        // A at depth 0 (dest)
        //   B at depth 1
        //   C at depth 1 (source, moved to be sibling of B)
        //     D at depth 2 (moving, stays as child of C)

        let slot_a = preview.slots.iter().find(|s| s.node_id == NodeId(0)).unwrap();
        let slot_b = preview.slots.iter().find(|s| s.node_id == NodeId(1)).unwrap();
        let slot_c = preview.slots.iter().find(|s| s.node_id == NodeId(2)).unwrap();
        let slot_d = preview.slots.iter().find(|s| s.node_id == NodeId(3)).unwrap();

        assert_eq!(slot_a.visual_depth, 0);
        assert_eq!(slot_a.role, NodeRole::Destination);

        assert_eq!(slot_b.visual_depth, 1);
        assert_eq!(slot_b.role, NodeRole::Normal);

        assert_eq!(slot_c.visual_depth, 1); // C moves to be sibling of B
        assert_eq!(slot_c.role, NodeRole::Source);

        // THIS IS THE KEY TEST: D should be depth 2 (child of C), not depth 1 (sibling of C)
        assert_eq!(slot_d.visual_depth, 2, "D should be at depth 2, child of C");
        assert_eq!(slot_d.role, NodeRole::Moving);
    }

    #[test]
    fn test_rebase_inline_mode_linear_chain() {
        // Test inline mode with a linear chain (the bug that was fixed)
        // Original tree (linear):
        // A (depth 0) <- dest
        //   B (depth 1) <- will be reparented to end of chain
        //     C (depth 2) <- source
        //       D (depth 3) <- descendant
        //
        // Rebase C with descendants onto A in INLINE mode
        // Expected: B goes to END of chain (after D), keeping tree linear
        // A (depth 0) <- dest
        //   C (depth 1) <- source
        //     D (depth 2) <- descendant
        //       B (depth 3) <- moved to end of chain
        let tree = make_tree(
            vec![
                make_node("aaaa", 0), // A (dest)
                make_node("bbbb", 1), // B (will be reparented to end)
                make_node("cccc", 2), // C (source)
                make_node("dddd", 3), // D (descendant)
            ],
            true,
        );

        let preview = PreviewBuilder::new(&tree).rebase_preview(
            NodeId(2), // source = C
            NodeId(0), // dest = A
            PreviewRebaseType::WithDescendants,
            false, // allow_branches = false (inline mode)
        );

        // Verify the chain is linear with B at the end
        let slot_a = preview.slots.iter().find(|s| s.node_id == NodeId(0)).unwrap();
        let slot_b = preview.slots.iter().find(|s| s.node_id == NodeId(1)).unwrap();
        let slot_c = preview.slots.iter().find(|s| s.node_id == NodeId(2)).unwrap();
        let slot_d = preview.slots.iter().find(|s| s.node_id == NodeId(3)).unwrap();

        assert_eq!(slot_a.visual_depth, 0, "A should be at depth 0 (dest)");
        assert_eq!(slot_c.visual_depth, 1, "C should be at depth 1 (source, child of A)");
        assert_eq!(slot_d.visual_depth, 2, "D should be at depth 2 (child of C)");
        assert_eq!(
            slot_b.visual_depth, 3,
            "B should be at depth 3 (END of chain, child of D)"
        );

        assert_eq!(slot_a.role, NodeRole::Destination);
        assert_eq!(slot_c.role, NodeRole::Source);
        assert_eq!(slot_d.role, NodeRole::Moving);
        assert_eq!(slot_b.role, NodeRole::Normal);
    }

    #[test]
    fn test_rebase_fork_mode() {
        // Test fork mode (allow_branches = true)
        // In fork mode, dest's children stay as siblings, not moved to end
        // Original tree:
        // A (depth 0) <- dest
        //   B (depth 1) <- stays as sibling
        //     C (depth 2) <- source
        //       D (depth 3) <- descendant
        //
        // Rebase C with descendants onto A in FORK mode
        // A (depth 0) <- dest
        //   B (depth 1) <- stays as child of A
        //   C (depth 1) <- source (sibling of B)
        //     D (depth 2) <- descendant
        let tree = make_tree(
            vec![
                make_node("aaaa", 0), // A
                make_node("bbbb", 1), // B
                make_node("cccc", 2), // C (source)
                make_node("dddd", 3), // D (descendant)
            ],
            true,
        );

        let preview = PreviewBuilder::new(&tree).rebase_preview(
            NodeId(2), // source = C
            NodeId(0), // dest = A
            PreviewRebaseType::WithDescendants,
            true, // allow_branches = true (fork mode)
        );

        let slot_b = preview.slots.iter().find(|s| s.node_id == NodeId(1)).unwrap();
        let slot_c = preview.slots.iter().find(|s| s.node_id == NodeId(2)).unwrap();
        let slot_d = preview.slots.iter().find(|s| s.node_id == NodeId(3)).unwrap();

        // In fork mode, B stays at depth 1 (sibling of C)
        assert_eq!(slot_b.visual_depth, 1, "B should be at depth 1 (child of A)");
        assert_eq!(slot_c.visual_depth, 1, "C should be at depth 1 (sibling of B)");
        assert_eq!(slot_d.visual_depth, 2, "D should be at depth 2 (child of C)");
    }
}
