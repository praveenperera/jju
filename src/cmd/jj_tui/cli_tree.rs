//! CLI rendering for tree output - mirrors TUI visual formatting

use super::handlers;
use super::state::DiffStats;
use super::tree::{TreeNode, TreeState};
use super::ui::format_bookmarks_truncated;
use super::vm::InlineRowBadge;
use colored::Colorize;

/// Print the tree to stdout with ANSI colors matching the TUI
pub fn print_tree(tree: &TreeState, full: bool) {
    for row in render_rows(tree, full) {
        println!("{row}");
    }
}

fn render_rows(tree: &TreeState, full: bool) -> Vec<String> {
    let mut hidden_count = 0;
    let mut depth_stack: Vec<usize> = Vec::new();
    let working_copy_stats = working_copy_stats(tree);
    let mut rows = Vec::new();

    for node in tree.nodes() {
        if !node.is_visible(full) {
            hidden_count += 1;
            continue;
        }

        while let Some(&parent_depth) = depth_stack.last() {
            if parent_depth < node.depth {
                break;
            }
            depth_stack.pop();
        }
        let visual_depth = depth_stack.len();
        depth_stack.push(node.depth);

        let show_hidden = if !full && hidden_count > 0 {
            let count = hidden_count;
            hidden_count = 0;
            count
        } else {
            hidden_count = 0;
            0
        };

        rows.push(format_row(
            node,
            visual_depth,
            show_hidden,
            inline_badge_for(node, working_copy_stats.as_ref()).as_ref(),
        ));
    }

    rows
}

fn working_copy_stats(tree: &TreeState) -> Option<DiffStats> {
    let working_copy = tree.nodes().iter().find(|node| node.is_working_copy)?;
    let output = crate::cmd::jj_tui::commands::diff::get_stats(&working_copy.change_id).ok()?;
    Some(handlers::diff::parse_diff_stats(&output))
}

fn format_row(
    node: &TreeNode,
    visual_depth: usize,
    hidden_count: usize,
    inline_badge: Option<&InlineRowBadge>,
) -> String {
    let indent = "  ".repeat(visual_depth);
    let connector = if visual_depth > 0 { "├── " } else { "" };
    let at_marker = if node.is_working_copy { "@ " } else { "" };

    let (prefix, suffix) = node
        .change_id
        .split_at(node.unique_prefix_len.min(node.change_id.len()));
    let colored_rev = format!("{}{}", prefix.purple(), suffix.dimmed());

    let count_str = if hidden_count > 0 {
        format!(" +{hidden_count}")
    } else {
        String::new()
    };

    let bookmark_str = if node.bookmarks.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            format_bookmarks_truncated(&node.bookmarks, 30).cyan()
        )
    };

    let inline_badge = format_inline_badge(inline_badge);

    let desc = if node.description.is_empty() {
        if node.is_working_copy {
            "(working copy)".dimmed().to_string()
        } else {
            "(no description)".dimmed().to_string()
        }
    } else {
        node.description.dimmed().to_string()
    };

    format!(
        "{indent}{connector}{at_marker}({colored_rev}){bookmark_str}{count_str}{inline_badge}  {desc}"
    )
}

fn inline_badge_for(
    node: &TreeNode,
    working_copy_stats: Option<&DiffStats>,
) -> Option<InlineRowBadge> {
    working_copy_stats
        .filter(|_| node.is_working_copy)
        .cloned()
        .map(InlineRowBadge::DiffStats)
        .or_else(|| node.is_empty.then_some(InlineRowBadge::EmptyRevision))
}

fn format_inline_badge(inline_badge: Option<&InlineRowBadge>) -> String {
    match inline_badge {
        Some(InlineRowBadge::DiffStats(stats)) => format!(
            "  {} {}",
            format!("+{}", stats.insertions).green(),
            format!("-{}", stats.deletions).red(),
        ),
        Some(InlineRowBadge::EmptyRevision) => format!("  {}", "∅".yellow()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{format_row, render_rows};
    use crate::cmd::jj_tui::state::DiffStats;
    use crate::cmd::jj_tui::test_support::{TestNodeKind, make_tree};
    use crate::cmd::jj_tui::vm::InlineRowBadge;

    #[test]
    fn renders_empty_marker_for_visible_empty_revision() {
        let mut node = TestNodeKind::Bookmarked(&["main"]).make_node("abcd", 0);
        node.is_empty = true;
        let tree = make_tree(vec![node]);

        let rows = render_rows(&tree, true);

        assert!(rows[0].contains("∅"));
    }

    #[test]
    fn renders_working_copy_inline_stats() {
        let mut node = TestNodeKind::Plain.make_node("@@@@", 0);
        node.is_working_copy = true;
        let row = format_row(
            &node,
            0,
            0,
            Some(&InlineRowBadge::DiffStats(DiffStats {
                files_changed: 1,
                insertions: 4,
                deletions: 2,
            })),
        );

        assert!(row.contains("+4"));
        assert!(row.contains("-2"));
    }
}
