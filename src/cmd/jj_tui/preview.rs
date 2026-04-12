mod ops;
mod slots;
#[cfg(test)]
mod test_support;

use super::tree::TreeState;

/// Unique identifier for a node in the preview tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Type of rebase operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewRebaseType {
    Single,
    WithDescendants,
}

/// Role of a node in the preview
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Normal,
    Source,
    Moving,
    Destination,
}

/// A slot in the preview display
#[derive(Debug, Clone)]
pub struct DisplaySlot {
    pub node_id: NodeId,
    pub visual_depth: usize,
    pub role: NodeRole,
}

/// Preview of tree state after operation
pub struct Preview {
    pub slots: Vec<DisplaySlot>,
    pub source_id: Option<NodeId>,
}

pub struct PreviewBuilder<'a> {
    tree: &'a TreeState,
}

impl<'a> PreviewBuilder<'a> {
    pub fn new(tree: &'a TreeState) -> Self {
        Self { tree }
    }

    pub fn rebase_preview(
        self,
        source: NodeId,
        dest: NodeId,
        rebase_type: PreviewRebaseType,
        allow_branches: bool,
    ) -> Preview {
        let visible_nodes = visible_node_indices(self.tree);
        if source == dest {
            return Preview {
                slots: slots::identity_slots(
                    &visible_nodes,
                    &visible_visual_depths(self.tree),
                    source,
                    dest,
                ),
                source_id: Some(source),
            };
        }

        let visible_topology = self.tree.topology.project_visible(&visible_nodes);
        let result = ops::apply_rebase_preview(
            visible_topology,
            ops::RebasePreviewOp {
                source,
                dest,
                rebase_type,
                allow_branches,
            },
        );

        Preview {
            slots: slots::project_slots(&result.topology, &result.moving_ids, source, dest),
            source_id: Some(source),
        }
    }
}

fn visible_node_indices(tree: &TreeState) -> Vec<usize> {
    tree.visible_entries
        .iter()
        .map(|entry| entry.node_index)
        .collect()
}

fn visible_visual_depths(tree: &TreeState) -> Vec<usize> {
    tree.visible_entries
        .iter()
        .map(|entry| entry.visual_depth)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::test_support::{find_slot, make_node, make_tree, visible_topology};
    use super::*;
    use crate::cmd::jj_tui::tree::{TreeState, TreeTopology, ViewMode, VisibleEntry};
    use ahash::HashSet;

    #[test]
    fn test_from_tree_linear() {
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
                make_node("dddd", 3),
            ],
            true,
        );

        let relations = visible_topology(&tree);
        assert_eq!(relations.parent_of(0), None);
        assert_eq!(relations.parent_of(1), Some(0));
        assert_eq!(relations.parent_of(2), Some(1));
        assert_eq!(relations.parent_of(3), Some(2));
        assert_eq!(relations.children_of(0), &[1]);
        assert_eq!(relations.children_of(1), &[2]);
        assert_eq!(relations.children_of(2), &[3]);
        assert_eq!(relations.children_of(3), &[] as &[usize]);
    }

    #[test]
    fn test_from_tree_forked() {
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
                make_node("dddd", 1),
            ],
            true,
        );

        let relations = visible_topology(&tree);
        assert_eq!(relations.parent_of(0), None);
        assert_eq!(relations.parent_of(1), Some(0));
        assert_eq!(relations.parent_of(2), Some(1));
        assert_eq!(relations.parent_of(3), Some(0));

        let children_of_a = relations.children_of(0);
        assert!(children_of_a.contains(&1));
        assert!(children_of_a.contains(&3));
        assert_eq!(children_of_a.len(), 2);
    }

    #[test]
    fn test_descendants() {
        let tree = make_tree(
            vec![
                make_node("aaaa", 0),
                make_node("bbbb", 1),
                make_node("cccc", 2),
            ],
            true,
        );

        let relations = visible_topology(&tree);
        let desc_a = relations.descendants(0);
        assert!(desc_a.contains(&1));
        assert!(desc_a.contains(&2));
        assert_eq!(desc_a.len(), 2);

        let desc_b = relations.descendants(1);
        assert!(desc_b.contains(&2));
        assert_eq!(desc_b.len(), 1);

        let desc_c = relations.descendants(2);
        assert!(desc_c.is_empty());
    }

    #[test]
    fn test_rebase_with_descendants() {
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
            NodeId(1),
            NodeId(0),
            PreviewRebaseType::WithDescendants,
            true,
        );

        assert_eq!(preview.slots.len(), 4);
        let slot_a = find_slot(&preview.slots, 0);
        let slot_b = find_slot(&preview.slots, 1);
        let slot_c = find_slot(&preview.slots, 2);
        let slot_d = find_slot(&preview.slots, 3);

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
        let nodes = vec![
            make_node("aaaa", 0),
            make_node("cccc", 2),
            make_node("dddd", 3),
        ];
        let visible_entries = vec![
            VisibleEntry {
                node_index: 0,
                visual_depth: 0,
                has_separator_before: false,
            },
            VisibleEntry {
                node_index: 1,
                visual_depth: 1,
                has_separator_before: false,
            },
            VisibleEntry {
                node_index: 2,
                visual_depth: 2,
                has_separator_before: false,
            },
        ];
        let topology = TreeTopology::from_nodes(&nodes);
        let tree = TreeState {
            nodes,
            topology,
            cursor: 0,
            scroll_offset: 0,
            full_mode: false,
            view_mode: ViewMode::Tree,
            expanded_entry: None,
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        };

        let relations = visible_topology(&tree);
        assert_eq!(relations.parent_of(0), None);
        assert_eq!(relations.parent_of(1), Some(0));
        assert_eq!(relations.parent_of(2), Some(1));
    }

    #[test]
    fn test_rebase_subtree_to_different_parent() {
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
            NodeId(2),
            NodeId(0),
            PreviewRebaseType::WithDescendants,
            true,
        );

        let slot_a = find_slot(&preview.slots, 0);
        let slot_b = find_slot(&preview.slots, 1);
        let slot_c = find_slot(&preview.slots, 2);
        let slot_d = find_slot(&preview.slots, 3);

        assert_eq!(slot_a.visual_depth, 0);
        assert_eq!(slot_a.role, NodeRole::Destination);
        assert_eq!(slot_b.visual_depth, 1);
        assert_eq!(slot_b.role, NodeRole::Normal);
        assert_eq!(slot_c.visual_depth, 1);
        assert_eq!(slot_c.role, NodeRole::Source);
        assert_eq!(slot_d.visual_depth, 2);
        assert_eq!(slot_d.role, NodeRole::Moving);
    }

    #[test]
    fn test_rebase_inline_mode_linear_chain() {
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
            NodeId(2),
            NodeId(0),
            PreviewRebaseType::WithDescendants,
            false,
        );

        let slot_a = find_slot(&preview.slots, 0);
        let slot_b = find_slot(&preview.slots, 1);
        let slot_c = find_slot(&preview.slots, 2);
        let slot_d = find_slot(&preview.slots, 3);

        assert_eq!(slot_a.visual_depth, 0);
        assert_eq!(slot_c.visual_depth, 1);
        assert_eq!(slot_d.visual_depth, 2);
        assert_eq!(slot_b.visual_depth, 3);
        assert_eq!(slot_a.role, NodeRole::Destination);
        assert_eq!(slot_c.role, NodeRole::Source);
        assert_eq!(slot_d.role, NodeRole::Moving);
        assert_eq!(slot_b.role, NodeRole::Normal);
    }

    #[test]
    fn test_rebase_fork_mode() {
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
            NodeId(2),
            NodeId(0),
            PreviewRebaseType::WithDescendants,
            true,
        );

        let slot_b = find_slot(&preview.slots, 1);
        let slot_c = find_slot(&preview.slots, 2);
        let slot_d = find_slot(&preview.slots, 3);

        assert_eq!(slot_b.visual_depth, 1);
        assert_eq!(slot_c.visual_depth, 1);
        assert_eq!(slot_d.visual_depth, 2);
    }
}
