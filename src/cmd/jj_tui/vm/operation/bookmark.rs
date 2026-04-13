use super::super::super::preview::NodeRole;
use super::super::row::Marker;
use super::OperationViewBuilder;

impl OperationViewBuilder<'_> {
    pub(in crate::cmd::jj_tui::vm) fn build_bookmark_move_view(
        &self,
        bookmark_name: &str,
        dest_cursor: usize,
    ) -> Vec<super::super::row::TreeRowVm> {
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
}
