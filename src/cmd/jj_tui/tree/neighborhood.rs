use super::{NeighborhoodState, TreeLoadScope, TreeSnapshot, TreeState, TreeViewState, ViewMode};

impl TreeState {
    pub fn is_neighborhood_mode(&self) -> bool {
        matches!(&self.view.view_mode, ViewMode::Neighborhood(..))
    }

    pub fn has_neighborhood_history(&self) -> bool {
        self.neighborhood_state()
            .map(|state| !state.history.is_empty())
            .unwrap_or(false)
    }

    pub fn current_entry_is_neighborhood_preview(&self) -> bool {
        self.current_entry()
            .and_then(|entry| entry.neighborhood.as_ref())
            .map(|entry| entry.is_preview)
            .unwrap_or(false)
    }

    pub fn enable_neighborhood(&mut self) {
        let Some(anchor_change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return;
        };
        self.view.load_scope = TreeLoadScope::Neighborhood;
        self.view.focus_stack.clear();
        self.set_view_mode(ViewMode::Neighborhood(NeighborhoodState::new(
            anchor_change_id.clone(),
        )));
        self.restore_cursor_to_change_id(&anchor_change_id);
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

    pub fn enter_neighborhood_path(&mut self) -> bool {
        let Some(target_change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return false;
        };
        let is_preview = self.current_entry_is_neighborhood_preview();
        let updated = if let Some(state) = self.neighborhood_state_mut() {
            if !is_preview || state.anchor_change_id == target_change_id {
                false
            } else {
                state.history.push(state.anchor_change_id.clone());
                state.anchor_change_id = target_change_id.clone();
                true
            }
        } else {
            false
        };
        if !updated {
            return false;
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&target_change_id);
        true
    }

    pub fn exit_neighborhood_path(&mut self) -> bool {
        let Some(anchor_change_id) = self
            .neighborhood_state_mut()
            .and_then(|state| state.history.pop())
        else {
            return false;
        };
        if let Some(state) = self.neighborhood_state_mut() {
            state.anchor_change_id = anchor_change_id.clone();
        }
        self.recompute_projection();
        self.restore_cursor_to_change_id(&anchor_change_id);
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
