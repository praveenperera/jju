mod neighborhood;
mod node;
mod snapshot;
mod view;

#[cfg(test)]
pub use neighborhood::NeighborhoodExtent;
pub use neighborhood::{NeighborhoodEntry, NeighborhoodResize, NeighborhoodState};
pub use node::{BookmarkInfo, DivergentVersion, TreeNode};
pub use snapshot::TreeSnapshot;
pub use view::{TreeLoadScope, TreeViewState, ViewMode, VisibleEntry};

pub struct TreeState {
    pub snapshot: TreeSnapshot,
    pub view: TreeViewState,
    pub projection: crate::cmd::jj_tui::tree::TreeProjection,
}
