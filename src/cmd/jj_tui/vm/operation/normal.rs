use super::super::super::preview::NodeRole;
use super::OperationViewBuilder;

impl OperationViewBuilder<'_> {
    pub(in crate::cmd::jj_tui::vm) fn build_normal_view(
        &self,
    ) -> Vec<super::super::row::TreeRowVm> {
        self.build_operation_view(self.app.tree.view.cursor, |_visible_idx, _node| {
            (NodeRole::Normal, None)
        })
    }
}
