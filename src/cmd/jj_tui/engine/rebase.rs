use super::super::state::{ModeState, RebaseType};
use super::super::tree::TreeState;
use ahash::{HashSet, HashSetExt};

pub fn compute_moving_indices(tree: &TreeState, mode: &ModeState) -> HashSet<usize> {
    let ModeState::Rebasing(state) = mode else {
        return HashSet::new();
    };

    let mut indices = HashSet::new();
    let mut in_source_tree = false;
    let mut source_struct_depth = 0usize;

    for (idx, entry) in tree.visible_entries().iter().enumerate() {
        let node = &tree.nodes()[entry.node_index];

        if node.change_id == state.source_rev {
            indices.insert(idx);
            if state.rebase_type == RebaseType::WithDescendants {
                in_source_tree = true;
                source_struct_depth = node.depth;
            }
        } else if in_source_tree {
            if node.depth > source_struct_depth {
                indices.insert(idx);
            } else {
                break;
            }
        }
    }

    indices
}
