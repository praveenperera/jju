use super::super::App;

pub(super) fn detail_hydration_order(app: &App) -> Vec<String> {
    let mut seen = ahash::HashSet::default();
    let mut commit_ids = Vec::new();

    if let Some(node) = app.tree.current_node()
        && seen.insert(node.commit_id.clone())
    {
        commit_ids.push(node.commit_id.clone());
    }

    for entry in app.tree.visible_entries() {
        let commit_id = app.tree.nodes()[entry.node_index].commit_id.clone();
        if seen.insert(commit_id.clone()) {
            commit_ids.push(commit_id);
        }
    }

    for node in app.tree.nodes() {
        if seen.insert(node.commit_id.clone()) {
            commit_ids.push(node.commit_id.clone());
        }
    }

    commit_ids
}
