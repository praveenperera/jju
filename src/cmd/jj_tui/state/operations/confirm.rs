use super::rebase::RebaseType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Abandon,
    StackSync,
    RebaseOntoTrunk(RebaseType),
    MoveBookmarkBackwards {
        bookmark_name: String,
        dest_rev: String,
    },
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub action: ConfirmAction,
    pub message: String,
    pub revs: Vec<String>,
}
