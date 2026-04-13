use super::super::{NeighborhoodResize, TreeState, ViewMode};

impl TreeState {
    pub fn expand_neighborhood(&mut self) -> NeighborhoodResize {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return NeighborhoodResize::NoChange;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return NeighborhoodResize::NoChange;
        };
        let resize = state.expand();
        self.sync_neighborhood_load_scope();

        if resize != NeighborhoodResize::Reprojected {
            return resize;
        }

        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        NeighborhoodResize::Reprojected
    }

    pub fn shrink_neighborhood(&mut self) -> NeighborhoodResize {
        let Some(change_id) = self.current_node().map(|node| node.change_id.clone()) else {
            return NeighborhoodResize::NoChange;
        };
        let ViewMode::Neighborhood(state) = &mut self.view.view_mode else {
            return NeighborhoodResize::NoChange;
        };
        let resize = state.shrink();
        self.sync_neighborhood_load_scope();

        if resize != NeighborhoodResize::Reprojected {
            return resize;
        }

        self.recompute_projection();
        self.restore_cursor_to_change_id(&change_id);
        NeighborhoodResize::Reprojected
    }
}
