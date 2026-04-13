use super::super::super::preview::NodeRole;
use super::super::row::Marker;
use super::OperationViewBuilder;

impl OperationViewBuilder<'_> {
    pub(in crate::cmd::jj_tui::vm) fn build_squash_view(
        &self,
        source_rev: &str,
        dest_cursor: usize,
    ) -> Vec<super::super::row::TreeRowVm> {
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
}
