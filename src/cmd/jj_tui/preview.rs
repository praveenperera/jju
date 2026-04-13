mod builder;
mod ops;
mod slots;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use self::builder::PreviewBuilder;

/// Unique identifier for a node in the preview tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Type of rebase operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewRebaseType {
    Single,
    WithDescendants,
}

/// Role of a node in the preview
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Normal,
    Source,
    Moving,
    Destination,
}

/// A slot in the preview display
#[derive(Debug, Clone)]
pub struct DisplaySlot {
    pub node_id: NodeId,
    pub visual_depth: usize,
    pub role: NodeRole,
}

/// Preview of tree state after operation
pub struct Preview {
    pub slots: Vec<DisplaySlot>,
    pub source_id: Option<NodeId>,
}
