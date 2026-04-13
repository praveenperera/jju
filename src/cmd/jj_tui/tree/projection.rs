use super::{
    TreeSnapshot, TreeViewState,
    neighborhood::{focused_root_index, neighborhood_state, resolve_neighborhood_anchor_index},
    visible::{self, NeighborhoodFilter, VisibleOptions},
};

#[derive(Clone, Debug)]
pub struct TreeProjection {
    pub visible_entries: Vec<super::VisibleEntry>,
}

impl TreeProjection {
    pub(in crate::cmd::jj_tui::tree) fn from_parts(
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
                    neighborhood_state(&view.view_mode).and_then(|state| {
                        let ancestor_limit = state.ancestor_limit()?;
                        let preview_depth_limit = state.preview_depth_limit()?;

                        Some(NeighborhoodFilter {
                            anchor_index,
                            ancestor_limit,
                            preview_depth_limit,
                        })
                    })
                }),
            },
        );
        Self { visible_entries }
    }
}
