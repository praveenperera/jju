pub(crate) struct StackSyncCommand {
    push: bool,
    force: bool,
}

impl StackSyncCommand {
    pub(crate) fn new(push: bool, force: bool) -> Self {
        Self { push, force }
    }

    pub(crate) fn run(self) -> eyre::Result<()> {
        jju_jj::stack_sync::StackSyncCommand::new(self.push, self.force).run()
    }
}
