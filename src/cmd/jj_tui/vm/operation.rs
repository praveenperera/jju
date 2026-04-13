use super::super::app::App;
use super::super::preview::{NodeId, NodeRole, PreviewBuilder, PreviewRebaseType};
use super::super::state::RebaseType;
use super::super::tree::TreeNode;
use super::details::build_row_details;
use super::row::{Marker, RowVmBuilder, TreeRowVm};

pub(super) struct OperationViewBuilder<'a> {
    app: &'a App,
}

impl<'a> OperationViewBuilder<'a> {
    pub(super) fn new(app: &'a App) -> Self {
        Self { app }
    }

    pub(super) fn build_normal_view(&self) -> Vec<TreeRowVm> {
        self.build_operation_view(self.app.tree.view.cursor, |_visible_idx, _node| {
            (NodeRole::Normal, None)
        })
    }

    pub(super) fn build_rebase_view(
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

    pub(super) fn build_bookmark_move_view(
        &self,
        bookmark_name: &str,
        dest_cursor: usize,
    ) -> Vec<TreeRowVm> {
        self.build_operation_view(dest_cursor, |visible_idx, node| {
            let is_source = node.has_bookmark(bookmark_name);
            let is_dest = visible_idx == dest_cursor && !is_source;

            if is_source {
                (NodeRole::Source, Some(Marker::Bookmark))
            } else if is_dest {
                (
                    NodeRole::Destination,
                    Some(Marker::Destination { mode_hint: None }),
                )
            } else {
                (NodeRole::Normal, None)
            }
        })
    }

    pub(super) fn build_squash_view(&self, source_rev: &str, dest_cursor: usize) -> Vec<TreeRowVm> {
        self.build_operation_view(dest_cursor, |visible_idx, node| {
            let is_source = node.change_id == source_rev;
            let is_dest = visible_idx == dest_cursor && !is_source;

            if is_source {
                (NodeRole::Source, Some(Marker::Source))
            } else if is_dest {
                (
                    NodeRole::Destination,
                    Some(Marker::Destination { mode_hint: None }),
                )
            } else {
                (NodeRole::Normal, None)
            }
        })
    }

    fn build_operation_view(
        &self,
        cursor_idx: usize,
        mut role_marker: impl FnMut(usize, &TreeNode) -> (NodeRole, Option<Marker>),
    ) -> Vec<TreeRowVm> {
        let is_expanded_mode = self.app.tree.view.expanded_entry.is_some();

        self.app
            .tree
            .visible_nodes()
            .enumerate()
            .map(|(visible_idx, entry)| {
                let node = self.app.tree.get_node(entry);
                let is_cursor = visible_idx == cursor_idx;
                let is_this_expanded = self.app.tree.is_expanded(visible_idx);
                let details = is_this_expanded.then(|| {
                    build_row_details(node, self.app.diff_stats_cache.get(&node.change_id))
                });
                let (role, marker) = role_marker(visible_idx, node);

                RowVmBuilder::new(node, entry.visual_depth)
                    .cursor(is_cursor)
                    .selected(self.app.tree.view.selected.contains(&visible_idx))
                    .dimmed(is_expanded_mode && !is_cursor && !is_this_expanded)
                    .zoom_root(self.app.tree.view.focus_stack.contains(&entry.node_index))
                    .role(role)
                    .marker(marker)
                    .details(details)
                    .separator_before(entry.has_separator_before)
                    .build()
            })
            .collect()
    }
}
