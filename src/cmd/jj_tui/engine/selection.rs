use super::super::tree::TreeState;

pub fn current_rev(tree: &TreeState) -> String {
    tree.current_node()
        .map(|node| node.change_id.clone())
        .unwrap_or_default()
}

pub fn get_revs_for_action(tree: &TreeState) -> Vec<String> {
    if tree.selected.is_empty() {
        vec![current_rev(tree)]
    } else {
        tree.selected
            .iter()
            .filter_map(|&idx| {
                tree.visible_entries
                    .get(idx)
                    .map(|entry| tree.nodes[entry.node_index].change_id.clone())
            })
            .collect()
    }
}

pub fn get_rev_at_cursor(tree: &TreeState, cursor: usize) -> Option<String> {
    tree.visible_entries
        .get(cursor)
        .map(|entry| tree.nodes[entry.node_index].change_id.clone())
}

pub fn extend_selection_to_cursor(tree: &mut TreeState) {
    if let Some(anchor) = tree.selection_anchor {
        tree.selected.clear();
        tree.select_range(anchor, tree.cursor);
    }
}
