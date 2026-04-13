use super::RevisionRunner;
use crate::cmd::jj_tui::runner::error;
use crate::cmd::jj_tui::state::MessageKind;

impl RevisionRunner<'_, '_> {
    pub(super) fn run_abandon(&mut self, revset: &str) {
        match crate::cmd::jj_tui::commands::revision::abandon(revset) {
            Ok(_) => {
                let count = revset.matches('|').count() + 1;
                if count == 1 {
                    self.0.success("Revision abandoned");
                } else {
                    self.0.success(format!("{count} revisions abandoned"));
                }
            }
            Err(error_value) => {
                let details = format!("{error_value}");
                self.0.set_status(
                    error::set_error_with_details("Abandon failed", &details),
                    MessageKind::Error,
                );
            }
        }
    }

    pub(super) fn run_resolve_divergence(
        &mut self,
        keep_commit_id: &str,
        abandon_commit_ids: Vec<String>,
    ) {
        let revset = abandon_commit_ids.join(" | ");
        match crate::cmd::jj_tui::commands::revision::abandon(&revset) {
            Ok(_) => {
                let count = abandon_commit_ids.len();
                let short_keep = &keep_commit_id[..keep_commit_id.len().min(8)];
                self.0.success(format!(
                    "Divergence resolved: kept {short_keep}, abandoned {count} version{}",
                    if count == 1 { "" } else { "s" }
                ));
            }
            Err(error_value) => {
                let details = format!("{error_value}");
                self.0.set_status(
                    error::set_error_with_details("Resolve divergence failed", &details),
                    MessageKind::Error,
                );
            }
        }
    }
}
