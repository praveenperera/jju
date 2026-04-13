use super::App;
use crate::jj_lib_helpers::JjRepo;

pub(super) fn load_node_details_sync(app: &mut App, commit_id: &str) {
    let Ok(jj_repo) = JjRepo::load(Some(&app.repo_path)) else {
        return;
    };
    let Ok(commit) = jj_repo.commit_by_id_hex(commit_id) else {
        return;
    };
    let Ok(details) = jj_repo.with_short_prefix_index(|prefix_index| {
        jj_repo.commit_details_with_index(&commit, prefix_index)
    }) else {
        return;
    };

    app.tree.hydrate_details(commit_id, details);
}
