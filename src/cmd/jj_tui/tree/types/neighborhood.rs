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
pub enum NeighborhoodExtent {
    Local(usize),
    FullTree,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeighborhoodState {
    pub anchor_change_id: String,
    pub history: Vec<String>,
    pub extent: NeighborhoodExtent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborhoodResize {
    NoChange,
    Reprojected,
    ScopeChanged,
}

impl NeighborhoodState {
    pub(in crate::cmd::jj_tui::tree) fn new(anchor_change_id: String) -> Self {
        Self {
            anchor_change_id,
            history: Vec::new(),
            extent: NeighborhoodExtent::Local(NEIGHBORHOOD_MIN_LEVEL),
        }
    }

    pub fn is_full_tree(&self) -> bool {
        matches!(self.extent, NeighborhoodExtent::FullTree)
    }

    pub fn local_level(&self) -> Option<usize> {
        match self.extent {
            NeighborhoodExtent::Local(level) => Some(level),
            NeighborhoodExtent::FullTree => None,
        }
    }

    pub fn ancestor_limit(&self) -> Option<usize> {
        self.local_level()
            .map(|level| NEIGHBORHOOD_BASE_ANCESTOR_LIMIT + level * NEIGHBORHOOD_ANCESTOR_STEP)
    }

    pub fn preview_depth_limit(&self) -> Option<usize> {
        self.local_level().map(|level| {
            NEIGHBORHOOD_BASE_PREVIEW_DEPTH_LIMIT + level * NEIGHBORHOOD_PREVIEW_DEPTH_STEP
        })
    }

    pub fn expand(&mut self) -> NeighborhoodResize {
        match self.extent {
            NeighborhoodExtent::Local(level) if level < NEIGHBORHOOD_MAX_LEVEL => {
                self.extent = NeighborhoodExtent::Local(level + 1);
                NeighborhoodResize::Reprojected
            }
            NeighborhoodExtent::Local(_) => {
                self.extent = NeighborhoodExtent::FullTree;
                NeighborhoodResize::ScopeChanged
            }
            NeighborhoodExtent::FullTree => NeighborhoodResize::NoChange,
        }
    }

    pub fn shrink(&mut self) -> NeighborhoodResize {
        match self.extent {
            NeighborhoodExtent::FullTree => {
                self.extent = NeighborhoodExtent::Local(NEIGHBORHOOD_MAX_LEVEL);
                NeighborhoodResize::ScopeChanged
            }
            NeighborhoodExtent::Local(level) if level > NEIGHBORHOOD_MIN_LEVEL => {
                self.extent = NeighborhoodExtent::Local(level - 1);
                NeighborhoodResize::Reprojected
            }
            NeighborhoodExtent::Local(_) => NeighborhoodResize::NoChange,
        }
    }
}
