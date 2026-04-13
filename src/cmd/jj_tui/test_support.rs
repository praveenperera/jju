use super::app::App;
use super::state::ModeState;
use super::tree::{
    BookmarkInfo, TreeLoadScope, TreeNode, TreeProjection, TreeSnapshot, TreeState, TreeTopology,
    TreeViewState, ViewMode, VisibleEntry,
};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Clone, Copy, Debug)]
pub(crate) enum TestNodeKind<'a> {
    Plain,
    Bookmarked(&'a [&'a str]),
}

impl TestNodeKind<'_> {
    pub(crate) fn make_node(self, change_id: &str, depth: usize) -> TreeNode {
        let bookmarks = match self {
            Self::Plain => vec![],
            Self::Bookmarked(bookmarks) => bookmarks
                .iter()
                .map(|&name| BookmarkInfo {
                    name: name.to_string(),
                    is_diverged: false,
                })
                .collect(),
        };

        TreeNode {
            change_id: change_id.to_string(),
            unique_prefix_len: 4,
            commit_id: format!("{change_id}000000"),
            description: String::new(),
            bookmarks,
            is_working_copy: false,
            has_conflicts: false,
            is_divergent: false,
            divergent_versions: vec![],
            parent_ids: vec![],
            depth,
            details: None,
        }
    }
}

pub(crate) fn make_tree(nodes: Vec<TreeNode>) -> TreeState {
    let visible_entries: Vec<VisibleEntry> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| VisibleEntry {
            node_index: index,
            visual_depth: node.depth,
            has_separator_before: false,
            neighborhood: None,
        })
        .collect();
    let topology = TreeTopology::from_nodes(&nodes);
    let snapshot = TreeSnapshot { nodes, topology };
    let view = TreeViewState {
        full_mode: true,
        view_mode: ViewMode::Tree,
        ..TreeViewState::new(TreeLoadScope::Stack)
    };
    let projection = TreeProjection { visible_entries };

    TreeState {
        snapshot,
        view,
        projection,
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
        last_op: None,
        syntax_set: SyntaxSet::load_defaults_newlines(),
        theme_set: ThemeSet::load_defaults(),
        repo_path: std::env::current_dir().unwrap_or_default(),
        detail_hydrator: None,
        detail_generation: 0,
    }
}
