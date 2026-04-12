use super::app::App;
use super::state::ModeState;
use super::tree::{BookmarkInfo, TreeNode, TreeState, TreeTopology, ViewMode, VisibleEntry};
use ahash::HashSet;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub(crate) fn make_node(change_id: &str, depth: usize) -> TreeNode {
    TreeNode {
        change_id: change_id.to_string(),
        unique_prefix_len: 4,
        commit_id: format!("{change_id}000000"),
        description: String::new(),
        bookmarks: vec![],
        is_working_copy: false,
        has_conflicts: false,
        is_divergent: false,
        divergent_versions: vec![],
        parent_ids: vec![],
        depth,
        details: None,
    }
}

pub(crate) fn make_node_with_bookmarks(
    change_id: &str,
    depth: usize,
    bookmarks: &[&str],
) -> TreeNode {
    let mut node = make_node(change_id, depth);
    node.bookmarks = bookmarks
        .iter()
        .map(|&name| BookmarkInfo {
            name: name.to_string(),
            is_diverged: false,
        })
        .collect();
    node
}

pub(crate) fn make_tree(nodes: Vec<TreeNode>) -> TreeState {
    let visible_entries: Vec<VisibleEntry> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| VisibleEntry {
            node_index: index,
            visual_depth: node.depth,
            has_separator_before: false,
        })
        .collect();
    let topology = TreeTopology::from_nodes(&nodes);

    TreeState {
        nodes,
        topology,
        cursor: 0,
        scroll_offset: 0,
        full_mode: true,
        view_mode: ViewMode::Tree,
        expanded_entry: None,
        visible_entries,
        selected: HashSet::default(),
        selection_anchor: None,
        focus_stack: Vec::new(),
    }
}

pub(crate) fn make_app_with_tree(tree: TreeState) -> App {
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
        repo_path: std::env::current_dir().unwrap_or_default(),
        detail_hydrator: None,
        detail_generation: 0,
    }
}
