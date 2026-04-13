use crate::cmd::jj_tui::{action::Action, controller::ControllerContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTemplate {
    Fixed(Action),
    NormalEnterConditional,
    PageUpHalfViewport,
    PageDownHalfViewport,
    CenterCursorViewport,
    BookmarkFilterChar,
    PushSelectFilterChar,
    NormalEscConditional,
}

impl ActionTemplate {
    pub(crate) fn build(&self, ctx: &ControllerContext<'_>, captured: Option<char>) -> Action {
        match self {
            ActionTemplate::Fixed(action) => action.clone(),
            ActionTemplate::NormalEnterConditional => {
                if ctx.can_enter_neighborhood_path {
                    Action::EnterNeighborhoodPath
                } else if ctx.neighborhood_active {
                    Action::Noop
                } else {
                    Action::ToggleFocus
                }
            }
            ActionTemplate::PageUpHalfViewport => Action::PageUp(ctx.viewport_height / 2),
            ActionTemplate::PageDownHalfViewport => Action::PageDown(ctx.viewport_height / 2),
            ActionTemplate::CenterCursorViewport => Action::CenterCursor(ctx.viewport_height),
            ActionTemplate::BookmarkFilterChar => {
                Action::BookmarkFilterChar(captured.unwrap_or(' '))
            }
            ActionTemplate::PushSelectFilterChar => {
                Action::PushSelectFilterChar(captured.unwrap_or(' '))
            }
            ActionTemplate::NormalEscConditional => {
                if ctx.has_focus {
                    Action::Unfocus
                } else if ctx.has_selection {
                    Action::ClearSelection
                } else if ctx.has_neighborhood_history {
                    Action::ExitNeighborhoodPath
                } else {
                    Action::Noop
                }
            }
        }
    }
}
