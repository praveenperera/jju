use crate::cmd::jj_tui::tree::{
    NeighborhoodAnchor, NeighborhoodState, TreeLoadScope, TreeState, ViewMode,
};

#[derive(Debug, Clone)]
pub(super) struct TreeRefreshRemapper {
    current_change_id: Option<String>,
    parent_change_id: Option<String>,
    old_cursor: usize,
    full_mode: bool,
    load_scope: TreeLoadScope,
    view_mode: ViewMode,
    focus_stack_change_ids: Vec<String>,
}

impl TreeRefreshRemapper {
    pub(super) fn capture(tree: &TreeState) -> Self {
        Self {
            current_change_id: tree.current_node().map(|node| node.change_id.clone()),
            parent_change_id: tree
                .current_node()
                .and_then(|node| node.parent_ids.first().cloned()),
            old_cursor: tree.view.cursor,
            full_mode: tree.view.full_mode,
            load_scope: tree.view.load_scope,
            view_mode: tree.view.view_mode.clone(),
            focus_stack_change_ids: tree
                .view
                .focus_stack
                .iter()
                .filter_map(|&index| tree.nodes().get(index).map(|node| node.change_id.clone()))
                .collect(),
        }
    }

    pub(super) fn load_scope(&self) -> TreeLoadScope {
        self.load_scope
    }

    pub(super) fn restore(self, tree: &mut TreeState) {
        tree.view.full_mode = self.full_mode;
        self.restore_mode(tree);
        self.restore_cursor(tree);

        if self.should_resume_follow_cursor() {
            tree.resume_neighborhood_follow_cursor();
        }
    }

    fn restore_mode(&self, tree: &mut TreeState) {
        match &self.view_mode {
            ViewMode::Tree => self.restore_focus_stack(tree),
            ViewMode::Neighborhood(state) => {
                tree.set_view_mode(ViewMode::Neighborhood(NeighborhoodState {
                    anchor: restored_anchor(state, self.current_change_id.as_deref()),
                    level: state.level,
                }))
            }
        }
    }

    fn restore_focus_stack(&self, tree: &mut TreeState) {
        for change_id in &self.focus_stack_change_ids {
            let Some(node_index) = find_node_index(tree, change_id) else {
                continue;
            };
            tree.focus_on(node_index);
        }
    }

    fn restore_cursor(&self, tree: &mut TreeState) {
        if let Some(change_id) = &self.current_change_id
            && let Some(index) = find_visible_index(tree, change_id)
        {
            tree.view.cursor = index;
            return;
        }

        if let Some(change_id) = &self.parent_change_id
            && let Some(index) = find_visible_index(tree, change_id)
        {
            tree.view.cursor = index;
            return;
        }

        tree.view.cursor = self.old_cursor.min(tree.visible_count().saturating_sub(1));
    }

    fn should_resume_follow_cursor(&self) -> bool {
        matches!(
            &self.view_mode,
            ViewMode::Neighborhood(NeighborhoodState {
                anchor: NeighborhoodAnchor::FollowCursor,
                ..
            })
        )
    }
}

fn restored_anchor(
    state: &NeighborhoodState,
    current_change_id: Option<&str>,
) -> NeighborhoodAnchor {
    match &state.anchor {
        NeighborhoodAnchor::FollowCursor => current_change_id
            .map(|change_id| NeighborhoodAnchor::Fixed(change_id.to_string()))
            .unwrap_or(NeighborhoodAnchor::FollowCursor),
        NeighborhoodAnchor::Fixed(change_id) => NeighborhoodAnchor::Fixed(change_id.clone()),
    }
}

fn find_node_index(tree: &TreeState, change_id: &str) -> Option<usize> {
    tree.nodes()
        .iter()
        .position(|node| node.change_id == change_id)
}

fn find_visible_index(tree: &TreeState, change_id: &str) -> Option<usize> {
    tree.visible_entries()
        .iter()
        .position(|entry| tree.nodes()[entry.node_index].change_id == change_id)
}

#[cfg(test)]
mod tests {
    use super::TreeRefreshRemapper;
    use crate::cmd::jj_tui::test_support::{TestNodeKind, make_tree};
    use crate::cmd::jj_tui::tree::{NeighborhoodAnchor, NeighborhoodState, ViewMode};

    #[test]
    fn restore_focus_stack_reapplies_focus_to_matching_change_ids() {
        let mut old_tree = make_tree(vec![
            TestNodeKind::Plain.make_node("a", 0),
            TestNodeKind::Plain.make_node("b", 1),
            TestNodeKind::Plain.make_node("c", 2),
        ]);
        old_tree.view.cursor = 1;
        old_tree.focus_on(1);
        let remapper = TreeRefreshRemapper::capture(&old_tree);

        let mut refreshed_tree = make_tree(vec![
            TestNodeKind::Plain.make_node("a", 0),
            TestNodeKind::Plain.make_node("b", 1),
            TestNodeKind::Plain.make_node("d", 2),
        ]);
        remapper.restore(&mut refreshed_tree);

        assert!(refreshed_tree.is_focused());
        assert_eq!(
            refreshed_tree
                .current_node()
                .map(|node| node.change_id.as_str()),
            Some("b")
        );
    }

    #[test]
    fn restore_neighborhood_follow_cursor_resumes_follow_mode() {
        let mut old_tree = make_tree(vec![
            TestNodeKind::Plain.make_node("a", 0),
            TestNodeKind::Plain.make_node("b", 1),
            TestNodeKind::Plain.make_node("c", 2),
        ]);
        old_tree.view.cursor = 1;
        old_tree.set_view_mode(ViewMode::Neighborhood(NeighborhoodState {
            anchor: NeighborhoodAnchor::FollowCursor,
            level: 2,
        }));
        let remapper = TreeRefreshRemapper::capture(&old_tree);

        let mut refreshed_tree = make_tree(vec![
            TestNodeKind::Plain.make_node("a", 0),
            TestNodeKind::Plain.make_node("b", 1),
            TestNodeKind::Plain.make_node("c", 2),
        ]);
        remapper.restore(&mut refreshed_tree);

        assert!(refreshed_tree.is_neighborhood_following_cursor());
        assert_eq!(
            refreshed_tree.neighborhood_state().map(|state| state.level),
            Some(2)
        );
        assert_eq!(
            refreshed_tree
                .current_node()
                .map(|node| node.change_id.as_str()),
            Some("b")
        );
    }

    #[test]
    fn restore_cursor_falls_back_to_parent_change() {
        let parent = TestNodeKind::Plain.make_node("parent", 0);
        let mut child = TestNodeKind::Plain.make_node("child", 1);
        child.parent_ids = vec!["parent".to_string()];

        let mut old_tree = make_tree(vec![parent.clone(), child]);
        old_tree.view.cursor = 1;
        let remapper = TreeRefreshRemapper::capture(&old_tree);

        let mut refreshed_tree = make_tree(vec![parent]);
        remapper.restore(&mut refreshed_tree);

        assert_eq!(
            refreshed_tree
                .current_node()
                .map(|node| node.change_id.as_str()),
            Some("parent")
        );
    }
}
