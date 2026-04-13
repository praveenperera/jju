use super::collect::TreeLoadInputs;
use super::graph::{build_nodes, ordered_roots};
use super::{CHANGE_ID_MIN_LEN, IdPrefixIndex, JjRepo, Result};
use crate::cmd::jj_tui::tree::TreeNode;

pub(super) fn assemble_tree_state(
    jj_repo: &JjRepo,
    prefix_index: &IdPrefixIndex,
    base: &str,
    inputs: &TreeLoadInputs,
) -> Result<Vec<TreeNode>> {
    let base_id = jj_repo.eval_revset_single(base).ok().and_then(|commit| {
        jj_repo
            .change_id_with_index(prefix_index, &commit, CHANGE_ID_MIN_LEN)
            .ok()
            .map(|(change_id, _)| change_id)
    });
    let roots = ordered_roots(
        &inputs.commit_map,
        &inputs.children_map,
        &inputs.working_copy_id,
        base_id.as_deref(),
    );

    Ok(build_nodes(
        &inputs.commit_map,
        &inputs.children_map,
        &roots,
    ))
}
