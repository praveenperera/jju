use super::super::super::{NeighborhoodEntry, TreeTopology, VisibleEntry};
use super::NeighborhoodProjection;
use ahash::HashMap;

pub(super) fn projected_entries(
    topology: &TreeTopology,
    projection: NeighborhoodProjection,
) -> Vec<VisibleEntry> {
    let projected = topology.project_visible(&projection.visible_nodes);
    let mut visual_depths: HashMap<usize, usize> = HashMap::default();
    let mut entries = Vec::with_capacity(projection.visible_nodes.len());
    let mut seen_root = false;

    for &node_index in &projection.visible_nodes {
        let visual_depth = projected
            .parent_of(node_index)
            .and_then(|parent_index| visual_depths.get(&parent_index).copied())
            .map_or(0, |depth| depth + 1);
        visual_depths.insert(node_index, visual_depth);

        let has_separator_before = visual_depth == 0 && seen_root;
        if visual_depth == 0 {
            seen_root = true;
        }

        entries.push(VisibleEntry {
            node_index,
            visual_depth,
            has_separator_before,
            neighborhood: projection.preview_nodes.contains(&node_index).then(|| {
                NeighborhoodEntry {
                    is_preview: true,
                    hidden_count: projection
                        .preview_hidden_counts
                        .get(&node_index)
                        .copied()
                        .unwrap_or_default(),
                }
            }),
        });
    }

    entries
}
