mod assemble;
mod collect;
mod divergence;
mod graph;
mod identity;
mod revset;

use super::{JjRepo, TreeLoadScope, TreeState};
use ahash::HashMap;
use assemble::assemble_tree_state;
use collect::collect_tree_inputs;
use eyre::Result;
use jj_lib::id_prefix::IdPrefixIndex;
use log::info;
use std::time::Instant;

const CHANGE_ID_MIN_LEN: usize = 4;

pub(super) fn load_tree_state(
    jj_repo: &JjRepo,
    base: &str,
    load_scope: TreeLoadScope,
) -> Result<TreeState> {
    let started_at = Instant::now();
    let working_copy = jj_repo.working_copy_commit()?;
    let revset = revset::revset_for_scope(base, load_scope);
    let commits = jj_repo.eval_revset(&revset)?;

    if commits.is_empty() {
        return Ok(TreeState::empty(load_scope));
    }

    jj_repo.with_short_prefix_index(|prefix_index| {
        let inputs = collect_tree_inputs(jj_repo, prefix_index, &commits, &working_copy)?;
        let nodes = assemble_tree_state(jj_repo, prefix_index, base, &inputs)?;

        info!(
            "Loaded tree summary for {} commits in {:?}",
            commits.len(),
            started_at.elapsed()
        );

        Ok(TreeState::from_nodes(nodes, load_scope))
    })
}

#[cfg(test)]
mod tests {
    use super::revset::revset_for_scope;
    use crate::cmd::jj_tui::tree::TreeLoadScope;

    #[test]
    fn stack_scope_keeps_capped_revset() {
        assert_eq!(
            revset_for_scope("trunk()", TreeLoadScope::Stack),
            "trunk() | ancestors(immutable_heads().., 2) | @::"
        );
    }

    #[test]
    fn neighborhood_scope_uses_uncapped_revset() {
        assert_eq!(
            revset_for_scope("trunk()", TreeLoadScope::Neighborhood),
            "trunk() | ancestors(immutable_heads()..) | @::"
        );
    }
}
