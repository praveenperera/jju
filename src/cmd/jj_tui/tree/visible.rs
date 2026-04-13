mod compact;
mod neighborhood;
#[cfg(test)]
mod tests;

use super::{TreeNode, TreeTopology, VisibleEntry};
use compact::{compact_mode_entries, full_mode_entries, visible_scope};
use neighborhood::neighborhood_entries;

pub(super) struct NeighborhoodFilter {
    pub anchor_index: usize,
    pub ancestor_limit: usize,
    pub descendant_limit: usize,
    pub sibling_depth_limit: usize,
}

pub(super) struct VisibleOptions {
    pub full_mode: bool,
    pub focused_root: Option<usize>,
    pub neighborhood: Option<NeighborhoodFilter>,
}

pub(super) fn compute_visible_entries(
    nodes: &[TreeNode],
    topology: &TreeTopology,
    options: VisibleOptions,
) -> Vec<VisibleEntry> {
    if let Some(neighborhood) = options.neighborhood {
        return neighborhood_entries(
            nodes.len(),
            topology,
            neighborhood.anchor_index,
            neighborhood.ancestor_limit,
            neighborhood.descendant_limit,
            neighborhood.sibling_depth_limit,
        );
    }

    let (filtered_nodes, base_depth) = visible_scope(nodes, topology, options.focused_root);

    if options.full_mode {
        full_mode_entries(&filtered_nodes, base_depth)
    } else {
        compact_mode_entries(&filtered_nodes, topology, options.full_mode)
    }
}
