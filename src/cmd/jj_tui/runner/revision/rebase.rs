use super::RevisionRunner;
use crate::cmd::jj_tui::runner::operations;
use crate::cmd::jj_tui::state::RebaseType;

impl RevisionRunner<'_, '_> {
    pub(super) fn run_rebase(
        &mut self,
        source: &str,
        dest: &str,
        rebase_type: RebaseType,
        allow_branches: bool,
    ) {
        let (text, kind) = operations::run_rebase(source, dest, rebase_type, allow_branches);
        self.0.set_status(text, kind);
    }

    pub(super) fn run_rebase_onto_trunk(&mut self, source: &str, rebase_type: RebaseType) {
        let (text, kind) = operations::run_rebase_onto_trunk(source, rebase_type);
        self.0.set_status(text, kind);
    }
}
