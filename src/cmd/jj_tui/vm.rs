//! View model for tree rows - separates state computation from rendering

use super::app::App;
use super::preview::{NodeId, NodeRole, PreviewBuilder, PreviewRebaseType};
use super::state::{DiffStats, ModeState, RebaseType};
use super::tree::{BookmarkInfo, TreeNode};

/// Pre-computed view state for a single tree row
#[derive(Debug, Clone)]
pub struct TreeRowVm {
    pub visual_depth: usize,

    // Role in current operation
    pub role: NodeRole,

    // Selection/cursor state
    pub is_cursor: bool,
    pub is_selected: bool,
    pub is_dimmed: bool,
    pub is_zoom_root: bool,
    pub is_working_copy: bool,
    pub has_conflicts: bool,
    pub is_divergent: bool,

    // Pre-formatted display data (NOT styled - just strings)
    pub change_id_prefix: String,
    pub change_id_suffix: String,
    pub bookmarks: Vec<BookmarkInfo>,
    pub description: String,

    // Marker text (e.g., "← src", "← dest")
    pub marker: Option<Marker>,

    // Optional expanded details
    pub details: Option<RowDetails>,

    // Visual height in terminal lines (1 for collapsed, more for expanded)
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct RowDetails {
    pub commit_id_prefix: String,
    pub commit_id_suffix: String,
    pub author: String,
    pub timestamp: String,
    pub full_description: String,
    pub diff_stats: Option<DiffStats>,
}

#[derive(Debug, Clone)]
pub enum Marker {
    Source,
    Destination { mode_hint: Option<String> },
    Moving,
    Bookmark,
}

/// Build the view model for all visible tree rows
pub fn build_tree_view(app: &App, _viewport_width: usize) -> Vec<TreeRowVm> {
    match &app.mode {
        ModeState::Rebasing(state) => build_rebase_view(
            app,
            &state.source_rev,
            state.dest_cursor,
            state.rebase_type,
            state.allow_branches,
        ),
        ModeState::MovingBookmark(state) => {
            build_bookmark_move_view(app, &state.bookmark_name, state.dest_cursor)
        }
        ModeState::Squashing(state) => build_squash_view(app, &state.source_rev, state.dest_cursor),
        _ => build_normal_view(app),
    }
}

/// Build view for normal mode (and selecting mode)
fn build_normal_view(app: &App) -> Vec<TreeRowVm> {
    let is_expanded_mode = app.tree.expanded_entry.is_some();

    app.tree
        .visible_nodes()
        .enumerate()
        .map(|(visible_idx, entry)| {
            let node = app.tree.get_node(entry);
            let is_cursor = visible_idx == app.tree.cursor;
            let is_this_expanded = app.tree.is_expanded(visible_idx);
            let is_dimmed = is_expanded_mode && !is_cursor && !is_this_expanded;

            let details = if is_this_expanded {
                Some(build_row_details(
                    node,
                    app.diff_stats_cache.get(&node.change_id),
                ))
            } else {
                None
            };

            build_row_vm(
                entry.visual_depth,
                node,
                is_cursor,
                app.tree.selected.contains(&visible_idx),
                is_dimmed,
                app.tree.focus_stack.contains(&entry.node_index),
                NodeRole::Normal,
                None,
                details,
            )
        })
        .collect()
}

/// Build view for rebase mode using preview system
fn build_rebase_view(
    app: &App,
    source_rev: &str,
    dest_cursor: usize,
    rebase_type: RebaseType,
    allow_branches: bool,
) -> Vec<TreeRowVm> {
    // Find source index from source_rev
    let source_idx = app
        .tree
        .visible_entries
        .iter()
        .enumerate()
        .find(|(_, entry)| app.tree.nodes[entry.node_index].change_id == source_rev)
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let preview_rebase_type = match rebase_type {
        RebaseType::Single => PreviewRebaseType::Single,
        RebaseType::WithDescendants => PreviewRebaseType::WithDescendants,
    };

    let preview = PreviewBuilder::new(&app.tree).rebase_preview(
        NodeId(source_idx),
        NodeId(dest_cursor),
        preview_rebase_type,
        allow_branches,
    );

    // Find cursor position in preview (the source node)
    let cursor_slot_idx = preview
        .source_id
        .and_then(|src| preview.slots.iter().position(|s| s.node_id == src));

    preview
        .slots
        .iter()
        .enumerate()
        .map(|(slot_idx, slot)| {
            let entry = &app.tree.visible_entries[slot.node_id.0];
            let node = &app.tree.nodes[entry.node_index];
            let is_cursor = cursor_slot_idx == Some(slot_idx);

            let marker = match slot.role {
                NodeRole::Source => Some(Marker::Source),
                NodeRole::Destination => Some(Marker::Destination {
                    mode_hint: Some(if allow_branches {
                        "fork".to_string()
                    } else {
                        "inline".to_string()
                    }),
                }),
                NodeRole::Moving => Some(Marker::Moving),
                _ => None,
            };

            build_row_vm(
                slot.visual_depth,
                node,
                is_cursor,
                false, // no multi-select in rebase mode
                false, // no dimming in rebase mode
                false, // no zoom markers in rebase mode
                slot.role,
                marker,
                None,
            )
        })
        .collect()
}

/// Build view for bookmark move mode
fn build_bookmark_move_view(app: &App, bookmark_name: &str, dest_cursor: usize) -> Vec<TreeRowVm> {
    let is_expanded_mode = app.tree.expanded_entry.is_some();

    app.tree
        .visible_nodes()
        .enumerate()
        .map(|(visible_idx, entry)| {
            let node = app.tree.get_node(entry);
            let is_source = node.has_bookmark(bookmark_name);
            let is_dest = visible_idx == dest_cursor && !is_source;
            let is_cursor = visible_idx == dest_cursor;
            let is_this_expanded = app.tree.is_expanded(visible_idx);
            let is_dimmed = is_expanded_mode && !is_cursor && !is_this_expanded;

            let role = if is_source {
                NodeRole::Source
            } else if is_dest {
                NodeRole::Destination
            } else {
                NodeRole::Normal
            };

            let marker = if is_source {
                Some(Marker::Bookmark)
            } else if is_dest {
                Some(Marker::Destination { mode_hint: None })
            } else {
                None
            };

            let details = if is_this_expanded {
                Some(build_row_details(
                    node,
                    app.diff_stats_cache.get(&node.change_id),
                ))
            } else {
                None
            };

            build_row_vm(
                entry.visual_depth,
                node,
                is_cursor,
                app.tree.selected.contains(&visible_idx),
                is_dimmed,
                app.tree.focus_stack.contains(&entry.node_index),
                role,
                marker,
                details,
            )
        })
        .collect()
}

/// Build view for squash mode
fn build_squash_view(app: &App, source_rev: &str, dest_cursor: usize) -> Vec<TreeRowVm> {
    let is_expanded_mode = app.tree.expanded_entry.is_some();

    app.tree
        .visible_nodes()
        .enumerate()
        .map(|(visible_idx, entry)| {
            let node = app.tree.get_node(entry);
            let is_source = node.change_id == source_rev;
            let is_dest = visible_idx == dest_cursor && !is_source;
            let is_cursor = visible_idx == dest_cursor;
            let is_this_expanded = app.tree.is_expanded(visible_idx);
            let is_dimmed = is_expanded_mode && !is_cursor && !is_this_expanded;

            let role = if is_source {
                NodeRole::Source
            } else if is_dest {
                NodeRole::Destination
            } else {
                NodeRole::Normal
            };

            let marker = if is_source {
                Some(Marker::Source)
            } else if is_dest {
                Some(Marker::Destination { mode_hint: None })
            } else {
                None
            };

            let details = if is_this_expanded {
                Some(build_row_details(
                    node,
                    app.diff_stats_cache.get(&node.change_id),
                ))
            } else {
                None
            };

            build_row_vm(
                entry.visual_depth,
                node,
                is_cursor,
                app.tree.selected.contains(&visible_idx),
                is_dimmed,
                app.tree.focus_stack.contains(&entry.node_index),
                role,
                marker,
                details,
            )
        })
        .collect()
}

/// Build details for an expanded row
fn build_row_details(node: &TreeNode, stats: Option<&DiffStats>) -> RowDetails {
    let author = if node.author_email.is_empty() {
        node.author_name.clone()
    } else {
        format!("{} <{}>", node.author_name, node.author_email)
    };

    let (commit_prefix, commit_suffix) = node
        .commit_id
        .split_at(node.unique_commit_prefix_len.min(node.commit_id.len()));

    RowDetails {
        commit_id_prefix: commit_prefix.to_string(),
        commit_id_suffix: commit_suffix.to_string(),
        author,
        timestamp: node.timestamp.clone(),
        full_description: node.full_description.clone(),
        diff_stats: stats.cloned(),
    }
}

/// Calculate visual height for a row based on its details
/// From render_commit_details_from_vm: 5 metadata lines + 1 description header + N description lines
fn row_height(details: &Option<RowDetails>) -> usize {
    match details {
        None => 1,
        Some(d) => {
            let desc_lines = d.full_description.trim().lines().count().max(1);
            1 + 5 + 1 + desc_lines // row + metadata (incl commit) + header + description
        }
    }
}

/// Build a single row view model
#[allow(clippy::too_many_arguments)]
fn build_row_vm(
    visual_depth: usize,
    node: &TreeNode,
    is_cursor: bool,
    is_selected: bool,
    is_dimmed: bool,
    is_zoom_root: bool,
    role: NodeRole,
    marker: Option<Marker>,
    details: Option<RowDetails>,
) -> TreeRowVm {
    let (prefix, suffix) = node
        .change_id
        .split_at(node.unique_prefix_len.min(node.change_id.len()));

    let description = if node.description.is_empty() {
        if node.is_working_copy {
            "(working copy)".to_string()
        } else {
            "(no description)".to_string()
        }
    } else {
        node.description.clone()
    };

    let height = row_height(&details);

    TreeRowVm {
        visual_depth,
        role,
        is_cursor,
        is_selected,
        is_dimmed,
        is_zoom_root,
        is_working_copy: node.is_working_copy,
        has_conflicts: node.has_conflicts,
        is_divergent: node.is_divergent,
        change_id_prefix: prefix.to_string(),
        change_id_suffix: suffix.to_string(),
        bookmarks: node.bookmarks.clone(),
        description,
        marker,
        details,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::tree::{TreeState, VisibleEntry};
    use ahash::{HashMap, HashSet};
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    fn make_node(change_id: &str, depth: usize) -> TreeNode {
        TreeNode {
            change_id: change_id.to_string(),
            unique_prefix_len: 4,
            commit_id: format!("{change_id}000000"),
            unique_commit_prefix_len: 7,
            description: String::new(),
            full_description: String::new(),
            bookmarks: vec![],
            is_working_copy: false,
            has_conflicts: false,
            is_divergent: false,
            divergent_versions: vec![],
            parent_ids: vec![],
            depth,
            author_name: String::new(),
            author_email: String::new(),
            timestamp: String::new(),
        }
    }

    fn make_tree(nodes: Vec<TreeNode>) -> TreeState {
        let visible_entries: Vec<VisibleEntry> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| VisibleEntry {
                node_index: i,
                visual_depth: n.depth,
            })
            .collect();

        TreeState {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            full_mode: true,
            expanded_entry: None,
            children_map: HashMap::default(),
            visible_entries,
            selected: HashSet::default(),
            selection_anchor: None,
            focus_stack: Vec::new(),
        }
    }

    fn make_app_with_tree(tree: TreeState) -> App {
        App {
            tree,
            mode: ModeState::Normal,
            should_quit: false,
            split_view: false,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_key: None,
            pending_operation: None,
            last_op: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    #[test]
    fn test_build_normal_view_cursor_tracking() {
        let tree = make_tree(vec![
            make_node("aaaa", 0),
            make_node("bbbb", 1),
            make_node("cccc", 2),
        ]);
        let mut app = make_app_with_tree(tree);
        app.tree.cursor = 1;

        let vms = build_tree_view(&app, 80);

        assert_eq!(vms.len(), 3);
        assert!(!vms[0].is_cursor);
        assert!(vms[1].is_cursor);
        assert!(!vms[2].is_cursor);
    }

    #[test]
    fn test_build_normal_view_selection_state() {
        let tree = make_tree(vec![
            make_node("aaaa", 0),
            make_node("bbbb", 1),
            make_node("cccc", 2),
        ]);
        let mut app = make_app_with_tree(tree);
        app.tree.selected.insert(0);
        app.tree.selected.insert(2);

        let vms = build_tree_view(&app, 80);

        assert!(vms[0].is_selected);
        assert!(!vms[1].is_selected);
        assert!(vms[2].is_selected);
    }

    #[test]
    fn test_build_rebase_view_roles() {
        use crate::cmd::jj_tui::state::RebaseState;

        let tree = make_tree(vec![
            make_node("aaaa", 0),
            make_node("bbbb", 1),
            make_node("cccc", 2),
        ]);
        let mut app = make_app_with_tree(tree);
        app.mode = ModeState::Rebasing(RebaseState {
            source_rev: "cccc".to_string(),
            dest_cursor: 0,
            rebase_type: RebaseType::Single,
            allow_branches: true,
            op_before: String::new(),
        });

        let vms = build_tree_view(&app, 80);

        // Find the source and dest
        let source_vm = vms.iter().find(|vm| vm.change_id_prefix == "cccc").unwrap();
        let dest_vm = vms.iter().find(|vm| vm.change_id_prefix == "aaaa").unwrap();

        assert_eq!(source_vm.role, NodeRole::Source);
        assert_eq!(dest_vm.role, NodeRole::Destination);
        assert!(matches!(source_vm.marker, Some(Marker::Source)));
        assert!(matches!(dest_vm.marker, Some(Marker::Destination { .. })));
    }
}
