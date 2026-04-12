#![cfg(test)]

use crate::cmd::jj_tui::preview::{DisplaySlot, NodeId};
use crate::cmd::jj_tui::tree::{
    TreeLoadScope, TreeNode, TreeState, TreeTopology, ViewMode, VisibleEntry,
};
use ahash::HashSet;

pub(super) fn make_node(change_id: &str, depth: usize) -> TreeNode {
    TreeNode {
        change_id: change_id.to_string(),
        unique_prefix_len: 4,
        commit_id: format!("{change_id}000000"),
        description: String::new(),
        bookmarks: vec![],
        is_working_copy: false,
        has_conflicts: false,
        is_divergent: false,
        divergent_versions: vec![],
        parent_ids: vec![],
        depth,
        details: None,
    }
}

pub(super) fn make_tree(nodes: Vec<TreeNode>, full_mode: bool) -> TreeState {
    let visible_entries: Vec<VisibleEntry> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| VisibleEntry {
            node_index: i,
            visual_depth: node.depth,
            has_separator_before: false,
        })
        .collect();
    let topology = TreeTopology::from_nodes(&nodes);

    TreeState {
        nodes,
        topology,
        cursor: 0,
        scroll_offset: 0,
        full_mode,
        load_scope: TreeLoadScope::Stack,
        view_mode: ViewMode::Tree,
        expanded_entry: None,
        visible_entries,
        selected: HashSet::default(),
        selection_anchor: None,
        focus_stack: Vec::new(),
    }
}

pub(super) fn visible_topology(tree: &TreeState) -> TreeTopology {
    let visible_nodes: Vec<usize> = tree
        .visible_entries
        .iter()
        .map(|entry| entry.node_index)
        .collect();
    tree.topology.project_visible(&visible_nodes)
}

pub(super) fn find_slot(slots: &[DisplaySlot], node_id: usize) -> &DisplaySlot {
    slots
        .iter()
        .find(|slot| slot.node_id == NodeId(node_id))
        .unwrap()
}
