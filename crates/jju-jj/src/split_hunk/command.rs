use super::{SplitHunkOptions, workflow::SplitHunkWorkflow};
use eyre::Result;

#[derive(Debug, Clone)]
pub struct SplitHunkCommand {
    options: SplitHunkOptions,
}

impl SplitHunkCommand {
    pub fn new(options: SplitHunkOptions) -> Self {
        Self { options }
    }

    pub fn run(self) -> Result<()> {
        SplitHunkWorkflow::new(self.options, super::repo::SplitHunkRepo).run()
    }
}
