#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackRootPlan {
    pub change_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackSyncPlan {
    pub trunk: String,
    pub roots: Vec<StackRootPlan>,
    pub push_bookmark_after_sync: bool,
}

impl StackSyncPlan {
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
}
