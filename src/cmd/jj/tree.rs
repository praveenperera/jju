use crate::cmd::jj_tui::{cli_tree, tree::TreeState};
use crate::jj_lib_helpers::JjRepo;
use eyre::Result;

pub(crate) struct TreeCommand {
    full: bool,
    from: Option<String>,
}

impl TreeCommand {
    pub(crate) fn new(full: bool, from: Option<String>) -> Self {
        Self { full, from }
    }

    pub(crate) fn run(self) -> Result<()> {
        let jj_repo = JjRepo::load(None)?;
        let base = self.from.as_deref().unwrap_or("trunk()");
        let tree = TreeState::load_with_base(&jj_repo, base)?;
        cli_tree::print_tree(&tree, self.full);
        Ok(())
    }
}
