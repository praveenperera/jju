use super::{NodeId, Preview, PreviewRebaseType, ops, slots};
use crate::cmd::jj_tui::tree::TreeState;

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

        let visible_topology = self.tree.snapshot.topology.project_visible(&visible_nodes);
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
    tree.visible_entries()
        .iter()
        .map(|entry| entry.node_index)
        .collect()
}

fn visible_visual_depths(tree: &TreeState) -> Vec<usize> {
    tree.visible_entries()
        .iter()
        .map(|entry| entry.visual_depth)
        .collect()
}
