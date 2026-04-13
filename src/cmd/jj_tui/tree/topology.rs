mod build;
mod mutate;
mod query;

use super::TreeNode;
use ahash::HashSet;

#[derive(Clone, Debug, Default)]
pub struct TreeTopology {
    parent: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
}

impl TreeTopology {
    pub fn from_nodes(nodes: &[TreeNode]) -> Self {
        build::from_nodes(nodes)
    }

    pub fn parent_of(&self, node_index: usize) -> Option<usize> {
        self.parent.get(node_index).copied().flatten()
    }

    pub fn children_of(&self, node_index: usize) -> &[usize] {
        self.children
            .get(node_index)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn roots(&self) -> &[usize] {
        &self.roots
    }

    pub fn descendants(&self, node_index: usize) -> HashSet<usize> {
        query::descendants(self, node_index)
    }

    pub fn subtree_nodes_in_order(&self, root: usize) -> Vec<usize> {
        query::subtree_nodes_in_order(self, root)
    }

    pub fn project_visible(&self, visible_nodes: &[usize]) -> Self {
        build::project_visible(self, visible_nodes)
    }

    pub fn remove_from_parent(&mut self, child: usize) {
        mutate::remove_from_parent(self, child);
    }

    pub fn add_child(&mut self, parent: usize, child: usize) {
        mutate::add_child(self, parent, child);
    }
}
