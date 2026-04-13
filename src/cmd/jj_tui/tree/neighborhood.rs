mod level;
mod mode;
mod path;

use super::{NeighborhoodState, TreeSnapshot, TreeState, TreeViewState, ViewMode};

impl TreeState {
    pub fn is_neighborhood_mode(&self) -> bool {
        matches!(&self.view.view_mode, ViewMode::Neighborhood(..))
    }

    pub fn has_neighborhood_history(&self) -> bool {
        self.neighborhood_state()
            .map(|state| !state.history.is_empty())
            .unwrap_or(false)
    }

    pub fn neighborhood_state(&self) -> Option<&NeighborhoodState> {
        neighborhood_state(&self.view.view_mode)
    }

    pub(super) fn restore_cursor_to_change_id(&mut self, change_id: &str) {
        if let Some(index) = self
            .projection
            .visible_entries
            .iter()
            .position(|entry| self.snapshot.nodes[entry.node_index].change_id == change_id)
        {
            self.view.cursor = index;
        }
    }

    fn neighborhood_state_mut(&mut self) -> Option<&mut NeighborhoodState> {
        match &mut self.view.view_mode {
            ViewMode::Neighborhood(state) => Some(state),
            ViewMode::Tree => None,
        }
    }
}

pub(super) fn focused_root_index(view: &TreeViewState) -> Option<usize> {
    if matches!(&view.view_mode, ViewMode::Neighborhood(..)) {
        None
    } else {
        view.focus_stack.last().copied()
    }
}

pub(super) fn neighborhood_state(view_mode: &ViewMode) -> Option<&NeighborhoodState> {
    match view_mode {
        ViewMode::Neighborhood(state) => Some(state),
        ViewMode::Tree => None,
    }
}

pub(super) fn resolve_neighborhood_anchor_index(
    snapshot: &TreeSnapshot,
    view: &TreeViewState,
    current_entry_node_index: Option<usize>,
) -> Option<usize> {
    let state = neighborhood_state(&view.view_mode)?;
    snapshot
        .nodes
        .iter()
        .position(|node| node.change_id == state.anchor_change_id)
        .or(current_entry_node_index)
}
