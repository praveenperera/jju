use super::super::super::TreeTopology;

pub(super) fn mainline_path(
    topology: &TreeTopology,
    anchor_index: usize,
    ancestor_limit: usize,
) -> Vec<usize> {
    let mut ancestors = Vec::new();
    let mut current = anchor_index;

    for _ in 0..ancestor_limit {
        let Some(parent_index) = topology.parent_of(current) else {
            break;
        };
        ancestors.push(parent_index);
        current = parent_index;
    }
    ancestors.reverse();

    let mut mainline = ancestors;
    mainline.push(anchor_index);

    let mut current = anchor_index;
    while let [child_index] = topology.children_of(current) {
        mainline.push(*child_index);
        current = *child_index;
    }

    mainline
}
