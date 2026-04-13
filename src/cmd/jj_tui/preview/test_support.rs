#![cfg(test)]

use crate::cmd::jj_tui::preview::{DisplaySlot, NodeId};
use crate::cmd::jj_tui::tree::{
    TreeLoadScope, TreeNode, TreeProjection, TreeSnapshot, TreeState, TreeTopology, TreeViewState,
    ViewMode, VisibleEntry,
};

pub(super) fn make_tree(nodes: Vec<TreeNode>, full_mode: bool) -> TreeState {
    let visible_entries: Vec<VisibleEntry> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| VisibleEntry {
            node_index: i,
            visual_depth: node.depth,
            has_separator_before: false,
            neighborhood: None,
        })
        .collect();
    let topology = TreeTopology::from_nodes(&nodes);
    let snapshot = TreeSnapshot { nodes, topology };
    let view = TreeViewState {
        full_mode,
        view_mode: ViewMode::Tree,
        ..TreeViewState::new(TreeLoadScope::Stack)
    };
    let projection = TreeProjection { visible_entries };

    TreeState {
        snapshot,
        view,
        projection,
    }
}

pub(super) fn visible_topology(tree: &TreeState) -> TreeTopology {
    let visible_nodes: Vec<usize> = tree
        .visible_entries()
        .iter()
        .map(|entry| entry.node_index)
        .collect();
    tree.snapshot.topology.project_visible(&visible_nodes)
}

pub(super) fn find_slot(slots: &[DisplaySlot], node_id: usize) -> &DisplaySlot {
    slots
        .iter()
        .find(|slot| slot.node_id == NodeId(node_id))
        .unwrap()
}
