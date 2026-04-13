mod hydration;
mod sync;

use super::App;
use crate::jj_lib_helpers::CommitDetails;
use std::sync::mpsc::Receiver;

struct DetailHydrationUpdate {
    generation: u64,
    commit_id: String,
    details: CommitDetails,
}

pub(crate) struct DetailHydrator {
    generation: u64,
    receiver: Receiver<DetailHydrationUpdate>,
}

impl App {
    pub fn ensure_expanded_row_data(&mut self) {
        if let Some(entry) = self.tree.current_entry()
            && self.tree.is_expanded(self.tree.view.cursor)
        {
            let (commit_id, change_id, needs_details) = {
                let node = &self.tree.nodes()[entry.node_index];
                (
                    node.commit_id.clone(),
                    node.change_id.clone(),
                    !node.has_details(),
                )
            };

            if needs_details {
                sync::load_node_details_sync(self, &commit_id);
            }
            let _ = self.get_diff_stats(&change_id);
        }
    }

    pub(super) fn start_detail_hydration(&mut self) {
        hydration::start_detail_hydration(self);
    }

    pub(super) fn apply_detail_updates(&mut self) {
        hydration::apply_detail_updates(self);
    }
}
