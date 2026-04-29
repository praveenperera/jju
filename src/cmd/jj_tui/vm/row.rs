use super::super::preview::NodeRole;
use super::super::state::DiffStats;
use super::super::tree::{BookmarkInfo, TreeNode};
use super::details::{RowDetails, row_height};

#[derive(Debug, Clone)]
pub struct TreeRowVm {
    pub visual_depth: usize,
    pub role: NodeRole,
    pub is_cursor: bool,
    pub is_selected: bool,
    pub is_dimmed: bool,
    pub is_zoom_root: bool,
    pub is_working_copy: bool,
    pub has_conflicts: bool,
    pub is_divergent: bool,
    pub change_id_prefix: String,
    pub change_id_suffix: String,
    pub bookmarks: Vec<BookmarkInfo>,
    pub description: String,
    pub inline_badge: Option<InlineRowBadge>,
    pub is_neighborhood_preview: bool,
    pub neighborhood_hidden_count: usize,
    pub marker: Option<Marker>,
    pub details: Option<RowDetails>,
    pub height: usize,
    pub has_separator_before: bool,
}

#[derive(Debug, Clone)]
pub enum Marker {
    Source,
    Destination { mode_hint: Option<String> },
    Moving,
    Bookmark,
}

#[derive(Debug, Clone)]
pub enum InlineRowBadge {
    EmptyRevision,
    DiffStats(DiffStats),
}

pub(super) struct RowVmBuilder<'a> {
    visual_depth: usize,
    node: &'a TreeNode,
    is_cursor: bool,
    is_selected: bool,
    is_dimmed: bool,
    is_zoom_root: bool,
    role: NodeRole,
    is_neighborhood_preview: bool,
    neighborhood_hidden_count: usize,
    marker: Option<Marker>,
    inline_diff_stats: Option<DiffStats>,
    details: Option<RowDetails>,
    has_separator_before: bool,
}

impl<'a> RowVmBuilder<'a> {
    pub(super) fn new(node: &'a TreeNode, visual_depth: usize) -> Self {
        Self {
            visual_depth,
            node,
            is_cursor: false,
            is_selected: false,
            is_dimmed: false,
            is_zoom_root: false,
            role: NodeRole::Normal,
            is_neighborhood_preview: false,
            neighborhood_hidden_count: 0,
            marker: None,
            inline_diff_stats: None,
            details: None,
            has_separator_before: false,
        }
    }

    pub(super) fn cursor(mut self, is_cursor: bool) -> Self {
        self.is_cursor = is_cursor;
        self
    }

    pub(super) fn selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    pub(super) fn dimmed(mut self, is_dimmed: bool) -> Self {
        self.is_dimmed = is_dimmed;
        self
    }

    pub(super) fn zoom_root(mut self, is_zoom_root: bool) -> Self {
        self.is_zoom_root = is_zoom_root;
        self
    }

    pub(super) fn role(mut self, role: NodeRole) -> Self {
        self.role = role;
        self
    }

    pub(super) fn neighborhood_preview(mut self, is_preview: bool, hidden_count: usize) -> Self {
        self.is_neighborhood_preview = is_preview;
        self.neighborhood_hidden_count = hidden_count;
        self
    }

    pub(super) fn marker(mut self, marker: Option<Marker>) -> Self {
        self.marker = marker;
        self
    }

    pub(super) fn inline_diff_stats(mut self, inline_diff_stats: Option<DiffStats>) -> Self {
        self.inline_diff_stats = inline_diff_stats;
        self
    }

    pub(super) fn details(mut self, details: Option<RowDetails>) -> Self {
        self.details = details;
        self
    }

    pub(super) fn separator_before(mut self, has_separator_before: bool) -> Self {
        self.has_separator_before = has_separator_before;
        self
    }

    pub(super) fn build(self) -> TreeRowVm {
        let (prefix, suffix) = self
            .node
            .change_id
            .split_at(self.node.unique_prefix_len.min(self.node.change_id.len()));
        let description = if self.node.description.is_empty() {
            if self.node.is_working_copy {
                "(working copy)".to_string()
            } else {
                "(no description)".to_string()
            }
        } else {
            self.node.description.clone()
        };
        let inline_badge = self
            .inline_diff_stats
            .map(InlineRowBadge::DiffStats)
            .or_else(|| self.node.is_empty.then_some(InlineRowBadge::EmptyRevision));

        TreeRowVm {
            visual_depth: self.visual_depth,
            role: self.role,
            is_cursor: self.is_cursor,
            is_selected: self.is_selected,
            is_dimmed: self.is_dimmed,
            is_zoom_root: self.is_zoom_root,
            is_working_copy: self.node.is_working_copy,
            has_conflicts: self.node.has_conflicts,
            is_divergent: self.node.is_divergent,
            change_id_prefix: prefix.to_string(),
            change_id_suffix: suffix.to_string(),
            bookmarks: self.node.bookmarks.clone(),
            description,
            inline_badge,
            is_neighborhood_preview: self.is_neighborhood_preview,
            neighborhood_hidden_count: self.neighborhood_hidden_count,
            marker: self.marker,
            height: row_height(self.details.as_ref()),
            details: self.details,
            has_separator_before: self.has_separator_before,
        }
    }
}
