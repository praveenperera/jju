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

    // Whether to render a blank separator line before this row
    pub has_separator_before: bool,
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
    build_operation_view(app, app.tree.cursor, |_visible_idx, _node| {
        (NodeRole::Normal, None)
    })
}

/// Build view for rebase mode using preview system
fn build_rebase_view(
    app: &App,
    source_rev: &str,
    dest_cursor: usize,
    rebase_type: RebaseType,
    allow_branches: bool,
) -> Vec<TreeRowVm> {
    let source_node_index = app
        .tree
        .visible_entries
        .iter()
        .find(|entry| app.tree.nodes[entry.node_index].change_id == source_rev)
        .map(|entry| entry.node_index)
        .unwrap_or(0);
    let dest_node_index = app
        .tree
        .visible_entries
        .get(dest_cursor)
        .map(|entry| entry.node_index)
        .unwrap_or(source_node_index);

    let preview_rebase_type = match rebase_type {
        RebaseType::Single => PreviewRebaseType::Single,
        RebaseType::WithDescendants => PreviewRebaseType::WithDescendants,
    };

    let preview = PreviewBuilder::new(&app.tree).rebase_preview(
        NodeId(source_node_index),
        NodeId(dest_node_index),
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
            let node = &app.tree.nodes[slot.node_id.0];
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

            build_row_vm(RowVmArgs {
                visual_depth: slot.visual_depth,
                node,
                is_cursor,
                is_selected: false,
                is_dimmed: false,
                is_zoom_root: false,
                role: slot.role,
                marker,
                details: None,
                has_separator_before: false,
            })
        })
        .collect()
}

/// Build view for bookmark move mode
fn build_bookmark_move_view(app: &App, bookmark_name: &str, dest_cursor: usize) -> Vec<TreeRowVm> {
    build_operation_view(app, dest_cursor, |visible_idx, node| {
        let is_source = node.has_bookmark(bookmark_name);
        let is_dest = visible_idx == dest_cursor && !is_source;

        if is_source {
            (NodeRole::Source, Some(Marker::Bookmark))
        } else if is_dest {
            (
                NodeRole::Destination,
                Some(Marker::Destination { mode_hint: None }),
            )
        } else {
            (NodeRole::Normal, None)
        }
    })
}

/// Build view for squash mode
fn build_squash_view(app: &App, source_rev: &str, dest_cursor: usize) -> Vec<TreeRowVm> {
    build_operation_view(app, dest_cursor, |visible_idx, node| {
        let is_source = node.change_id == source_rev;
        let is_dest = visible_idx == dest_cursor && !is_source;

        if is_source {
            (NodeRole::Source, Some(Marker::Source))
        } else if is_dest {
            (
                NodeRole::Destination,
                Some(Marker::Destination { mode_hint: None }),
            )
        } else {
            (NodeRole::Normal, None)
        }
    })
}

fn build_operation_view(
    app: &App,
    cursor_idx: usize,
    mut role_marker: impl FnMut(usize, &TreeNode) -> (NodeRole, Option<Marker>),
) -> Vec<TreeRowVm> {
    let is_expanded_mode = app.tree.expanded_entry.is_some();

    app.tree
        .visible_nodes()
        .enumerate()
        .map(|(visible_idx, entry)| {
            let node = app.tree.get_node(entry);
            let is_cursor = visible_idx == cursor_idx;
            let is_this_expanded = app.tree.is_expanded(visible_idx);
            let is_dimmed = is_expanded_mode && !is_cursor && !is_this_expanded;
            let (role, marker) = role_marker(visible_idx, node);

            let details = if is_this_expanded {
                Some(build_row_details(
                    node,
                    app.diff_stats_cache.get(&node.change_id),
                ))
            } else {
                None
            };

            build_row_vm(RowVmArgs {
                visual_depth: entry.visual_depth,
                node,
                is_cursor,
                is_selected: app.tree.selected.contains(&visible_idx),
                is_dimmed,
                is_zoom_root: app.tree.focus_stack.contains(&entry.node_index),
                role,
                marker,
                details,
                has_separator_before: entry.has_separator_before,
            })
        })
        .collect()
}

/// Build details for an expanded row
fn build_row_details(node: &TreeNode, stats: Option<&DiffStats>) -> RowDetails {
    let Some(details) = node.details.as_ref() else {
        let split_at = node.commit_id.len().min(12);
        let (commit_id_prefix, commit_id_suffix) = node.commit_id.split_at(split_at);

        return RowDetails {
            commit_id_prefix: commit_id_prefix.to_string(),
            commit_id_suffix: commit_id_suffix.to_string(),
            author: "loading...".to_string(),
            timestamp: "loading...".to_string(),
            full_description: "loading...".to_string(),
            diff_stats: stats.cloned(),
        };
    };

    let author = if details.author_email.is_empty() {
        details.author_name.clone()
    } else {
        format!("{} <{}>", details.author_name, details.author_email)
    };

    let (commit_prefix, commit_suffix) = node
        .commit_id
        .split_at(details.unique_commit_prefix_len.min(node.commit_id.len()));

    RowDetails {
        commit_id_prefix: commit_prefix.to_string(),
        commit_id_suffix: commit_suffix.to_string(),
        author,
        timestamp: details.timestamp.clone(),
        full_description: details.full_description.clone(),
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

struct RowVmArgs<'a> {
    visual_depth: usize,
    node: &'a TreeNode,
    is_cursor: bool,
    is_selected: bool,
    is_dimmed: bool,
    is_zoom_root: bool,
    role: NodeRole,
    marker: Option<Marker>,
    details: Option<RowDetails>,
    has_separator_before: bool,
}

/// Build a single row view model
fn build_row_vm(args: RowVmArgs<'_>) -> TreeRowVm {
    let (prefix, suffix) = args
        .node
        .change_id
        .split_at(args.node.unique_prefix_len.min(args.node.change_id.len()));

    let description = if args.node.description.is_empty() {
        if args.node.is_working_copy {
            "(working copy)".to_string()
        } else {
            "(no description)".to_string()
        }
    } else {
        args.node.description.clone()
    };

    let height = row_height(&args.details);

    TreeRowVm {
        visual_depth: args.visual_depth,
        role: args.role,
        is_cursor: args.is_cursor,
        is_selected: args.is_selected,
        is_dimmed: args.is_dimmed,
        is_zoom_root: args.is_zoom_root,
        is_working_copy: args.node.is_working_copy,
        has_conflicts: args.node.has_conflicts,
        is_divergent: args.node.is_divergent,
        change_id_prefix: prefix.to_string(),
        change_id_suffix: suffix.to_string(),
        bookmarks: args.node.bookmarks.clone(),
        description,
        marker: args.marker,
        details: args.details,
        height,
        has_separator_before: args.has_separator_before,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::jj_tui::test_support::{make_app_with_tree, make_node, make_tree};
    use crate::jj_lib_helpers::CommitDetails;

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

    #[test]
    fn test_build_row_details_uses_loading_placeholder_while_pending() {
        let node = make_node("aaaa", 0);

        let details = build_row_details(&node, None);

        assert_eq!(details.author, "loading...");
        assert_eq!(details.timestamp, "loading...");
        assert_eq!(details.full_description, "loading...");
    }

    #[test]
    fn test_build_row_details_uses_hydrated_commit_metadata() {
        let mut node = make_node("aaaa", 0);
        node.commit_id = "1234567890abcdef".to_string();
        node.details = Some(CommitDetails {
            unique_commit_prefix_len: 8,
            full_description: "full body".to_string(),
            author_name: "Praveen".to_string(),
            author_email: "praveen@example.com".to_string(),
            timestamp: "2 days ago".to_string(),
        });

        let details = build_row_details(&node, None);

        assert_eq!(details.commit_id_prefix, "12345678");
        assert_eq!(details.commit_id_suffix, "90abcdef");
        assert_eq!(details.author, "Praveen <praveen@example.com>");
        assert_eq!(details.timestamp, "2 days ago");
        assert_eq!(details.full_description, "full body");
    }
}
