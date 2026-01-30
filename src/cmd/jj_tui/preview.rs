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
            // INLINE mode: source takes over dest's children
            // dest's children become source's children
            for &child in &dest_children {
                if !moving_ids.contains(&child) {
                    self.relations.remove_from_parent(child);
                    self.relations.add_child(source, child);
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

    spans.extend([
        Span::raw(format!("{indent}{connector}{at_marker}(")),
        Span::styled(prefix.to_string(), Style::default().fg(prefix_color)),
        Span::styled(suffix.to_string(), Style::default().fg(Color::DarkGray)),
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
    spans.push(Span::styled(
        format!("  {desc}"),
        Style::default().fg(Color::DarkGray),
    ));

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
