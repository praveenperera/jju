use super::super::super::preview::{NodeId, NodeRole, PreviewBuilder, PreviewRebaseType};
use super::super::super::state::RebaseType;
use super::super::row::{Marker, RowVmBuilder, TreeRowVm};
use super::OperationViewBuilder;

impl OperationViewBuilder<'_> {
    pub(in crate::cmd::jj_tui::vm) fn build_rebase_view(
        &self,
        source_rev: &str,
        dest_cursor: usize,
        rebase_type: RebaseType,
        allow_branches: bool,
    ) -> Vec<TreeRowVm> {
        let source_node_index = self
            .app
            .tree
            .visible_entries()
            .iter()
            .find(|entry| self.app.tree.nodes()[entry.node_index].change_id == source_rev)
            .map(|entry| entry.node_index)
            .unwrap_or(0);
        let dest_node_index = self
            .app
            .tree
            .visible_entries()
            .get(dest_cursor)
            .map(|entry| entry.node_index)
            .unwrap_or(source_node_index);
        let preview_rebase_type = match rebase_type {
            RebaseType::Single => PreviewRebaseType::Single,
            RebaseType::WithDescendants => PreviewRebaseType::WithDescendants,
        };
        let preview = PreviewBuilder::new(&self.app.tree).rebase_preview(
            NodeId(source_node_index),
            NodeId(dest_node_index),
            preview_rebase_type,
            allow_branches,
        );
        let cursor_slot_idx = preview
            .source_id
            .and_then(|src| preview.slots.iter().position(|slot| slot.node_id == src));

        preview
            .slots
            .iter()
            .enumerate()
            .map(|(slot_idx, slot)| {
                let node = &self.app.tree.nodes()[slot.node_id.0];
                let marker = match slot.role {
                    NodeRole::Source => Some(Marker::Source),
                    NodeRole::Destination => Some(Marker::Destination {
                        mode_hint: Some(if allow_branches {
                            "fork".to_string()
                        } else {
                            "inline".to_string()
                        }),
                    }),
                    NodeRole::Moving => Some(Marker::Moving),
                    _ => None,
                };

                RowVmBuilder::new(node, slot.visual_depth)
                    .cursor(cursor_slot_idx == Some(slot_idx))
                    .role(slot.role)
                    .marker(marker)
                    .build()
            })
            .collect()
    }
}
