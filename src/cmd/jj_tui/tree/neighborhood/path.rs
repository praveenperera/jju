use super::super::TreeState;

impl TreeState {
    pub fn current_entry_is_neighborhood_preview(&self) -> bool {
        self.current_entry()
            .and_then(|entry| entry.neighborhood.as_ref())
            .map(|entry| entry.is_preview)
            .unwrap_or(false)
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
}
