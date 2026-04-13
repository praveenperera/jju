use super::{
    NeighborhoodAnchor, NeighborhoodState, TreeLoadScope, TreeSnapshot, TreeState, TreeViewState,
    ViewMode,
};

impl TreeState {
    pub fn is_neighborhood_mode(&self) -> bool {
        matches!(&self.view.view_mode, ViewMode::Neighborhood(..))
    }

    pub fn is_neighborhood_following_cursor(&self) -> bool {
        matches!(
            &self.view.view_mode,
            ViewMode::Neighborhood(NeighborhoodState {
                anchor: NeighborhoodAnchor::FollowCursor,
                ..
            })
        )
    }

    pub fn enable_neighborhood(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        self.view.load_scope = TreeLoadScope::Neighborhood;
        self.view.focus_stack.clear();
        self.set_view_mode(ViewMode::Neighborhood(NeighborhoodState::follow_cursor()));
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn disable_neighborhood(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        self.view.load_scope = TreeLoadScope::Stack;
        self.set_view_mode(ViewMode::Tree);
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn toggle_neighborhood(&mut self) {
        if self.is_neighborhood_mode() {
            self.disable_neighborhood();
        } else {
            self.enable_neighborhood();
        }
    }

    pub fn freeze_neighborhood_anchor(&mut self) {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return;
        };
        if let ViewMode::Neighborhood(state) = &mut self.view.view_mode {
            state.anchor = NeighborhoodAnchor::Fixed(change_id);
        }
    }

    pub fn resume_neighborhood_follow_cursor(&mut self) {
        let anchor_change_id = self.current_node().map(|node| node.change_id.clone());
        if let ViewMode::Neighborhood(state) = &mut self.view.view_mode {
            state.anchor = NeighborhoodAnchor::FollowCursor;
        } else {
            return;
        }

        self.recompute_projection();
        if let Some(change_id) = anchor_change_id {
            self.restore_cursor_to_change_id(&change_id);
        }
    }

    pub fn neighborhood_state(&self) -> Option<&NeighborhoodState> {
        neighborhood_state(&self.view.view_mode)
    }

    pub fn expand_neighborhood(&mut self) -> bool {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return false;
        };
        if !state.expand() {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        true
    }

    pub fn shrink_neighborhood(&mut self) -> bool {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return false;
        };
        if !state.shrink() {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        true
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

    pub(super) fn sync_neighborhood_to_cursor(&mut self) {
        if !self.is_neighborhood_following_cursor() {
            return;
        }

        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return;
        };

        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
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
    match &state.anchor {
        NeighborhoodAnchor::FollowCursor => current_entry_node_index
            .or_else(|| snapshot.nodes.iter().position(|node| node.is_working_copy)),
        NeighborhoodAnchor::Fixed(change_id) => snapshot
            .nodes
            .iter()
            .position(|node| node.change_id == *change_id)
            .or(current_entry_node_index),
    }
}
