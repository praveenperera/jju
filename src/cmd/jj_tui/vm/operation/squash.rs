use std::collections::HashSet;

use super::super::super::preview::NodeRole;
use super::super::row::Marker;
use super::OperationViewBuilder;

impl OperationViewBuilder<'_> {
    pub(in crate::cmd::jj_tui::vm) fn build_squash_view(
        &self,
        source_revs: &[String],
        dest_cursor: usize,
    ) -> Vec<super::super::row::TreeRowVm> {
        let source_revs: HashSet<&str> = source_revs.iter().map(String::as_str).collect();
        self.build_operation_view(dest_cursor, |visible_idx, node| {
            let is_source = source_revs.contains(node.change_id.as_str());
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
