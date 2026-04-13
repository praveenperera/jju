mod bookmark;
mod normal;
mod rebase;
mod shared;
mod squash;

use super::super::app::App;

pub(super) struct OperationViewBuilder<'a> {
    app: &'a App,
}

impl<'a> OperationViewBuilder<'a> {
    pub(super) fn new(app: &'a App) -> Self {
        Self { app }
    }
}
