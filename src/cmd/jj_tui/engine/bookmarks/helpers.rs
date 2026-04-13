use crate::cmd::jj_tui::{commands::is_ancestor, tree::TreeState};
use ahash::{HashSet, HashSetExt};

pub(in crate::cmd::jj_tui::engine::bookmarks) fn is_bookmark_move_backwards(
    tree: &TreeState,
    bookmark_name: &str,
    dest_rev: &str,
) -> bool {
    let Some(current_node) = tree
        .nodes()
        .iter()
        .find(|node| node.has_bookmark(bookmark_name))
    else {
        return false;
    };

    is_ancestor(dest_rev, &current_node.change_id).unwrap_or(false)
}

pub(in crate::cmd::jj_tui::engine::bookmarks) fn bookmark_is_on_rev(
    tree: &TreeState,
    bookmark_name: &str,
    rev: &str,
) -> bool {
    tree.nodes()
        .iter()
        .any(|node| node.change_id == rev && node.has_bookmark(bookmark_name))
}

pub(in crate::cmd::jj_tui::engine::bookmarks) fn build_move_bookmark_picker_list(
    all_bookmarks: Vec<String>,
    pinned: Vec<String>,
    tree: &TreeState,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut pinned_unique = Vec::new();
    for name in pinned {
        if seen.insert(name.clone()) {
            pinned_unique.push(name);
        }
    }

    let pinned_set = seen;
    let mut rest: Vec<String> = all_bookmarks
        .into_iter()
        .filter(|bookmark| !pinned_set.contains(bookmark))
        .collect();
    sort_bookmarks_by_proximity(&mut rest, tree);

    let mut ordered = pinned_unique;
    ordered.extend(rest);
    ordered
}

pub(in crate::cmd::jj_tui::engine::bookmarks) fn previous_char_boundary(
    text: &str,
    cursor: usize,
) -> usize {
    text[..cursor]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn sort_bookmarks_by_proximity(bookmarks: &mut [String], tree: &TreeState) {
    let bookmark_indices = tree.bookmark_to_visible_index();
    let cursor = tree.view.cursor;

    bookmarks.sort_by(|left, right| {
        let left_index = bookmark_indices.get(left).copied();
        let right_index = bookmark_indices.get(right).copied();

        match (left_index, right_index) {
            (None, None) => left.cmp(right),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(left_index), Some(right_index)) => {
                let left_above = left_index < cursor;
                let right_above = right_index < cursor;

                match (left_above, right_above) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let left_distance = left_index.abs_diff(cursor);
                        let right_distance = right_index.abs_diff(cursor);
                        left_distance
                            .cmp(&right_distance)
                            .then_with(|| left.cmp(right))
                    }
                }
            }
        }
    });
}
