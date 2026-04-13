use super::TreeTopology;

pub(super) fn remove_from_parent(topology: &mut TreeTopology, child: usize) {
    if let Some(old_parent) = topology.parent.get_mut(child).and_then(Option::take) {
        if let Some(siblings) = topology.children.get_mut(old_parent) {
            siblings.retain(|&node_index| node_index != child);
        }
        if !topology.roots.contains(&child) {
            topology.roots.push(child);
        }
    }
}

pub(super) fn add_child(topology: &mut TreeTopology, parent: usize, child: usize) {
    if let Some(existing_parent) = topology.parent_of(child)
        && existing_parent == parent
    {
        return;
    }

    topology.parent[child] = Some(parent);
    topology.children[parent].push(child);
    topology.roots.retain(|&node_index| node_index != child);
}
