use super::TreeNode;
use ahash::{HashSet, HashSetExt};

#[derive(Clone, Debug, Default)]
pub struct TreeTopology {
    parent: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
}

impl TreeTopology {
    pub fn from_nodes(nodes: &[TreeNode]) -> Self {
        let mut parent = vec![None; nodes.len()];
        let mut children = vec![Vec::new(); nodes.len()];
        let mut roots = Vec::new();
        let mut depth_stack: Vec<(usize, usize)> = Vec::new();

        for (node_index, node) in nodes.iter().enumerate() {
            while let Some(&(parent_depth, _)) = depth_stack.last() {
                if parent_depth < node.depth {
                    break;
                }
                depth_stack.pop();
            }

            if let Some((_, parent_index)) = depth_stack.last().copied() {
                parent[node_index] = Some(parent_index);
                children[parent_index].push(node_index);
            } else {
                roots.push(node_index);
            }

            depth_stack.push((node.depth, node_index));
        }

        Self {
            parent,
            children,
            roots,
        }
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
        let mut result = HashSet::new();
        let mut stack = self.children_of(node_index).to_vec();

        while let Some(current) = stack.pop() {
            if result.insert(current) {
                stack.extend(self.children_of(current).iter().copied());
            }
        }

        result
    }

    pub fn subtree_nodes_in_order(&self, root: usize) -> Vec<usize> {
        let mut nodes = Vec::new();
        self.collect_subtree(root, &mut nodes);
        nodes
    }

    fn collect_subtree(&self, node_index: usize, nodes: &mut Vec<usize>) {
        nodes.push(node_index);
        for &child in self.children_of(node_index) {
            self.collect_subtree(child, nodes);
        }
    }

    pub fn project_visible(&self, visible_nodes: &[usize]) -> Self {
        let mut parent = vec![None; self.parent.len()];
        let mut children = vec![Vec::new(); self.children.len()];
        let mut roots = Vec::new();
        let visible_set: HashSet<usize> = visible_nodes.iter().copied().collect();

        for &node_index in visible_nodes {
            let mut current_parent = self.parent_of(node_index);
            while let Some(parent_index) = current_parent {
                if visible_set.contains(&parent_index) {
                    parent[node_index] = Some(parent_index);
                    children[parent_index].push(node_index);
                    break;
                }
                current_parent = self.parent_of(parent_index);
            }

            if parent[node_index].is_none() {
                roots.push(node_index);
            }
        }

        Self {
            parent,
            children,
            roots,
        }
    }

    pub fn remove_from_parent(&mut self, child: usize) {
        if let Some(old_parent) = self.parent.get_mut(child).and_then(Option::take) {
            if let Some(siblings) = self.children.get_mut(old_parent) {
                siblings.retain(|&node_index| node_index != child);
            }
            if !self.roots.contains(&child) {
                self.roots.push(child);
            }
        }
    }

    pub fn add_child(&mut self, parent: usize, child: usize) {
        if let Some(existing_parent) = self.parent_of(child)
            && existing_parent == parent
        {
            return;
        }

        self.parent[child] = Some(parent);
        self.children[parent].push(child);
        self.roots.retain(|&node_index| node_index != child);
    }
}
