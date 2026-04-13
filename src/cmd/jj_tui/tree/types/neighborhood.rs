const NEIGHBORHOOD_MIN_LEVEL: usize = 0;
const NEIGHBORHOOD_MAX_LEVEL: usize = 6;
const NEIGHBORHOOD_BASE_ANCESTOR_LIMIT: usize = 4;
const NEIGHBORHOOD_BASE_PREVIEW_DEPTH_LIMIT: usize = 2;
const NEIGHBORHOOD_ANCESTOR_STEP: usize = 4;
const NEIGHBORHOOD_PREVIEW_DEPTH_STEP: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborhoodEntry {
    pub is_preview: bool,
    pub hidden_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborhoodState {
    pub anchor_change_id: String,
    pub history: Vec<String>,
    pub level: usize,
}

impl NeighborhoodState {
    pub(in crate::cmd::jj_tui::tree) fn new(anchor_change_id: String) -> Self {
        Self {
            anchor_change_id,
            history: Vec::new(),
            level: NEIGHBORHOOD_MIN_LEVEL,
        }
    }

    pub fn ancestor_limit(&self) -> usize {
        NEIGHBORHOOD_BASE_ANCESTOR_LIMIT + self.level * NEIGHBORHOOD_ANCESTOR_STEP
    }

    pub fn preview_depth_limit(&self) -> usize {
        NEIGHBORHOOD_BASE_PREVIEW_DEPTH_LIMIT + self.level * NEIGHBORHOOD_PREVIEW_DEPTH_STEP
    }

    pub fn expand(&mut self) -> bool {
        if self.level >= NEIGHBORHOOD_MAX_LEVEL {
            return false;
        }
        self.level += 1;
        true
    }

    pub fn shrink(&mut self) -> bool {
        if self.level == NEIGHBORHOOD_MIN_LEVEL {
            return false;
        }
        self.level -= 1;
        true
    }
}
