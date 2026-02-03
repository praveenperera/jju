//! CLI rendering for tree output - mirrors TUI visual formatting

use super::tree::{BookmarkInfo, TreeState};
use super::ui::format_bookmarks_truncated;
use colored::Colorize;

/// Print the tree to stdout with ANSI colors matching the TUI
pub fn print_tree(tree: &TreeState, full: bool) {
    // Iterate in structural order (already traversed depth-first in TreeState)
    // Track which nodes are visible and compute hidden counts between them
    let mut last_visible_depth: Option<usize> = None;
    let mut hidden_count = 0;
    let mut depth_stack: Vec<usize> = Vec::new(); // tracks structural depths for visual depth calc

    for node in &tree.nodes {
        let is_visible = full || !node.bookmarks.is_empty() || node.is_working_copy;

        if !is_visible {
            hidden_count += 1;
            continue;
        }

        // Compute visual depth based on visible ancestors
        // Pop stack until we find an ancestor (node with smaller structural depth)
        while let Some(&parent_depth) = depth_stack.last() {
            if parent_depth < node.depth {
                break;
            }
            depth_stack.pop();
        }
        let visual_depth = depth_stack.len();
        depth_stack.push(node.depth);

        // Determine hidden count to show (only show when there's a gap from last visible)
        let show_hidden = if !full && hidden_count > 0 {
            let count = hidden_count;
            hidden_count = 0;
            count
        } else {
            hidden_count = 0;
            0
        };

        print_row(node, visual_depth, show_hidden);
        last_visible_depth = Some(node.depth);
    }

    // Suppress unused variable warning
    let _ = last_visible_depth;
}

fn print_row(node: &super::tree::TreeNode, visual_depth: usize, hidden_count: usize) {
    let indent = "  ".repeat(visual_depth);
    let connector = if visual_depth > 0 { "├── " } else { "" };
    let at_marker = if node.is_working_copy { "@ " } else { "" };

    // Format change ID with colored prefix (matches TUI purple/magenta)
    let (prefix, suffix) = node
        .change_id
        .split_at(node.unique_prefix_len.min(node.change_id.len()));
    let colored_rev = format!("{}{}", prefix.purple(), suffix.dimmed());

    // Hidden count indicator
    let count_str = if hidden_count > 0 {
        format!(" +{hidden_count}")
    } else {
        String::new()
    };

    // Format bookmarks (cyan, matching TUI)
    let bookmark_str = if node.bookmarks.is_empty() {
        String::new()
    } else {
        format!(" {}", format_bookmarks_for_cli(&node.bookmarks, 30).cyan())
    };

    // Description (dimmed, matching TUI)
    let desc = if node.description.is_empty() {
        if node.is_working_copy {
            "(working copy)".dimmed().to_string()
        } else {
            "(no description)".dimmed().to_string()
        }
    } else {
        node.description.dimmed().to_string()
    };

    println!("{indent}{connector}{at_marker}({colored_rev}){bookmark_str}{count_str}  {desc}");
}

/// Format bookmarks for CLI output, similar to ui::format_bookmarks_truncated
/// but returns a plain string for colored crate
fn format_bookmarks_for_cli(bookmarks: &[BookmarkInfo], max_width: usize) -> String {
    format_bookmarks_truncated(bookmarks, max_width)
}
