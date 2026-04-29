use super::super::super::preview::NodeRole;
use super::super::super::tree::TreeNode;
use super::super::details::build_row_details;
use super::super::row::{Marker, RowVmBuilder, TreeRowVm};
use super::OperationViewBuilder;

impl OperationViewBuilder<'_> {
    pub(super) fn build_operation_view(
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
                let inline_diff_stats = if is_cursor || node.is_working_copy {
                    self.app.diff_stats_cache.get(&node.change_id).cloned()
                } else {
                    None
                };
                let details = is_this_expanded.then(|| {
                    build_row_details(node, self.app.diff_stats_cache.get(&node.change_id))
                });
                let (role, marker) = role_marker(visible_idx, node);
                let neighborhood = entry.neighborhood.as_ref();

                RowVmBuilder::new(node, entry.visual_depth)
                    .cursor(is_cursor)
                    .selected(self.app.tree.view.selected.contains(&visible_idx))
                    .dimmed(is_expanded_mode && !is_cursor && !is_this_expanded)
                    .zoom_root(self.app.tree.view.focus_stack.contains(&entry.node_index))
                    .role(role)
                    .neighborhood_preview(
                        neighborhood.map(|entry| entry.is_preview).unwrap_or(false),
                        neighborhood
                            .map(|entry| entry.hidden_count)
                            .unwrap_or_default(),
                    )
                    .marker(marker)
                    .inline_diff_stats(inline_diff_stats)
                    .details(details)
                    .separator_before(entry.has_separator_before)
                    .build()
            })
            .collect()
    }
}
