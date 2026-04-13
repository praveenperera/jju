use super::node::TreeNode;
use crate::cmd::jj_tui::tree::TreeTopology;

#[derive(Clone, Debug)]
pub struct TreeSnapshot {
    pub nodes: Vec<TreeNode>,
    pub topology: TreeTopology,
}

impl TreeSnapshot {
    pub(in crate::cmd::jj_tui::tree) fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            topology: TreeTopology::default(),
        }
    }

    pub(crate) fn from_nodes(nodes: Vec<TreeNode>) -> Self {
        let topology = TreeTopology::from_nodes(&nodes);
        Self { nodes, topology }
    }
}
