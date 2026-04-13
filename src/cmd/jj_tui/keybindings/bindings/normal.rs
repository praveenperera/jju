use super::super::ActionTemplate::{
    CenterCursorViewport, NormalEscConditional, PageDownHalfViewport, PageUpHalfViewport,
};
use super::super::ModeId::Normal;
use super::super::{BindingBehavior, BindingSpec, CommandSpec, KeyDef};
use super::{chord, fixed, pending_prefix, single};
use crate::cmd::jj_tui::action::Action;
use crate::cmd::jj_tui::state::{BookmarkSelectAction, RebaseType};
use ratatui::crossterm::event::KeyCode;

pub(super) fn commands() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new(
            Normal,
            "quit",
            fixed(Action::Quit),
            vec![single(KeyDef::Char('q')), single(KeyDef::Ctrl('c'))],
        )
        .help("General", "Quit"),
        CommandSpec::new(
            Normal,
            "squash",
            fixed(Action::EnterSquashMode),
            vec![single(KeyDef::Char('Q'))],
        )
        .help("Rebase", "Squash into target"),
        CommandSpec::new(
            Normal,
            "esc",
            BindingBehavior::Action(NormalEscConditional),
            vec![single(KeyDef::Key(KeyCode::Esc))],
        ),
        CommandSpec::new(
            Normal,
            "help",
            fixed(Action::EnterHelp),
            vec![single(KeyDef::Char('?'))],
        )
        .help("General", "Toggle help"),
        CommandSpec::new(
            Normal,
            "down",
            fixed(Action::MoveCursorDown),
            vec![
                single(KeyDef::Char('j')),
                single(KeyDef::Key(KeyCode::Down)),
            ],
        )
        .help("Navigation", "Move cursor down")
        .help_aliases(),
        CommandSpec::new(
            Normal,
            "up",
            fixed(Action::MoveCursorUp),
            vec![single(KeyDef::Char('k')), single(KeyDef::Key(KeyCode::Up))],
        )
        .help("Navigation", "Move cursor up")
        .help_aliases(),
        CommandSpec::new(
            Normal,
            "working_copy",
            fixed(Action::JumpToWorkingCopy),
            vec![single(KeyDef::Char('@'))],
        )
        .help("Navigation", "Jump to working copy"),
        CommandSpec::new(
            Normal,
            "page_up",
            BindingBehavior::Action(PageUpHalfViewport),
            vec![single(KeyDef::Ctrl('u'))],
        )
        .help("Navigation", "Page up"),
        CommandSpec::new(
            Normal,
            "page_down",
            BindingBehavior::Action(PageDownHalfViewport),
            vec![single(KeyDef::Ctrl('d'))],
        )
        .help("Navigation", "Page down"),
        CommandSpec::new(
            Normal,
            "git",
            pending_prefix("git"),
            vec![single(KeyDef::Char('g'))],
        ),
        CommandSpec::new(
            Normal,
            "nav",
            pending_prefix("nav"),
            vec![single(KeyDef::Char('z'))],
        ),
        CommandSpec::new(
            Normal,
            "bookmark",
            pending_prefix("bookmark"),
            vec![single(KeyDef::Char('b'))],
        ),
        CommandSpec::new(
            Normal,
            "full",
            fixed(Action::ToggleFullMode),
            vec![single(KeyDef::Char('f'))],
        )
        .help("View", "Toggle full mode"),
        CommandSpec::new(
            Normal,
            "zoom",
            fixed(Action::ToggleFocus),
            vec![single(KeyDef::Key(KeyCode::Enter))],
        )
        .help("Navigation", "Zoom in/out on node"),
        CommandSpec::new(
            Normal,
            "details",
            fixed(Action::ToggleExpanded),
            vec![single(KeyDef::Key(KeyCode::Tab)), single(KeyDef::Char(' '))],
        )
        .help("View", "Toggle commit details")
        .help_aliases(),
        CommandSpec::new(
            Normal,
            "split",
            fixed(Action::ToggleSplitView),
            vec![single(KeyDef::Char('\\'))],
        )
        .help("View", "Toggle split view"),
        CommandSpec::new(
            Normal,
            "refresh",
            fixed(Action::RefreshTree),
            vec![single(KeyDef::Char('R'))],
        )
        .help("View", "Refresh tree"),
        CommandSpec::new(
            Normal,
            "diff",
            fixed(Action::EnterDiffView),
            vec![single(KeyDef::Char('d'))],
        )
        .help("View", "View diff"),
        CommandSpec::new(
            Normal,
            "desc",
            fixed(Action::EditDescription),
            vec![single(KeyDef::Char('D'))],
        )
        .help("View", "Edit description"),
        CommandSpec::new(
            Normal,
            "edit",
            fixed(Action::EditWorkingCopy),
            vec![single(KeyDef::Char('e'))],
        )
        .help("Edit Operations", "Edit working copy (jj edit)"),
        BindingSpec::new(
            Normal,
            "new",
            fixed(Action::CreateNewCommit),
            vec![single(KeyDef::Char('n'))],
        )
        .help("Edit Operations", "New commit (jj new)"),
        BindingSpec::new(
            Normal,
            "commit",
            fixed(Action::CommitWorkingCopy),
            vec![single(KeyDef::Char('c'))],
        )
        .help("Edit Operations", "Commit changes (jj commit)"),
        BindingSpec::new(
            Normal,
            "toggle",
            fixed(Action::ToggleSelection),
            vec![single(KeyDef::Char('x'))],
        )
        .help("Selection", "Toggle selection"),
        BindingSpec::new(
            Normal,
            "select",
            fixed(Action::EnterSelecting),
            vec![single(KeyDef::Char('v'))],
        )
        .help("Selection", "Visual select mode"),
        BindingSpec::new(
            Normal,
            "abandon",
            fixed(Action::EnterConfirmAbandon),
            vec![single(KeyDef::Char('a'))],
        )
        .help("Selection", "Abandon selected"),
        BindingSpec::new(
            Normal,
            "rebase_single",
            fixed(Action::EnterRebaseMode(RebaseType::Single)),
            vec![single(KeyDef::Char('r'))],
        )
        .help("Rebase", "Rebase single (-r)"),
        BindingSpec::new(
            Normal,
            "rebase_desc",
            fixed(Action::EnterRebaseMode(RebaseType::WithDescendants)),
            vec![single(KeyDef::Char('s'))],
        )
        .help("Rebase", "Rebase + descendants (-s)"),
        BindingSpec::new(
            Normal,
            "trunk_single",
            fixed(Action::EnterConfirmRebaseOntoTrunk(RebaseType::Single)),
            vec![single(KeyDef::Char('t'))],
        )
        .help("Rebase", "Quick rebase onto trunk"),
        BindingSpec::new(
            Normal,
            "trunk_desc",
            fixed(Action::EnterConfirmRebaseOntoTrunk(
                RebaseType::WithDescendants,
            )),
            vec![single(KeyDef::Char('T'))],
        )
        .help("Rebase", "Quick rebase tree onto trunk"),
        BindingSpec::new(
            Normal,
            "undo",
            fixed(Action::Undo),
            vec![single(KeyDef::Char('u'))],
        )
        .help("Rebase", "Undo last operation"),
        BindingSpec::new(
            Normal,
            "push",
            fixed(Action::GitPush),
            vec![single(KeyDef::Char('p'))],
        )
        .help("Bookmarks & Git", "Push current bookmark"),
        BindingSpec::new(
            Normal,
            "push_all",
            fixed(Action::GitPushAll),
            vec![single(KeyDef::Char('P'))],
        ),
        BindingSpec::new(
            Normal,
            "stack_sync",
            fixed(Action::EnterConfirmStackSync),
            vec![single(KeyDef::Char('S'))],
        )
        .help("Bookmarks & Git", "Stack sync (fetch, rebase, clean up)"),
        BindingSpec::new(
            Normal,
            "conflicts",
            fixed(Action::EnterConflicts),
            vec![single(KeyDef::Char('C'))],
        )
        .help("Conflicts", "View conflicts panel"),
        BindingSpec::new(
            Normal,
            "fetch",
            fixed(Action::GitFetch),
            vec![chord('g', KeyDef::Char('f'))],
        )
        .help("Bookmarks & Git", "Git fetch")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "import",
            fixed(Action::GitImport),
            vec![chord('g', KeyDef::Char('i'))],
        )
        .help("Bookmarks & Git", "Git import")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "export",
            fixed(Action::GitExport),
            vec![chord('g', KeyDef::Char('e'))],
        )
        .help("Bookmarks & Git", "Git export")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "resolve_divergence",
            fixed(Action::ResolveDivergence),
            vec![chord('g', KeyDef::Char('r'))],
        )
        .help("Bookmarks & Git", "Resolve divergence (keep local)")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "create_pr",
            fixed(Action::CreatePR),
            vec![chord('g', KeyDef::Char('p'))],
        )
        .help("Bookmarks & Git", "Create/open PR from bookmark")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "top",
            fixed(Action::MoveCursorTop),
            vec![chord('z', KeyDef::Char('t'))],
        )
        .help("Navigation", "Jump to top")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "bottom",
            fixed(Action::MoveCursorBottom),
            vec![chord('z', KeyDef::Char('b'))],
        )
        .help("Navigation", "Jump to bottom")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "center",
            BindingBehavior::Action(CenterCursorViewport),
            vec![chord('z', KeyDef::Char('z'))],
        )
        .help("Navigation", "Center current line")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "neighborhood",
            fixed(Action::ToggleNeighborhood),
            vec![chord('z', KeyDef::Char('n'))],
        )
        .help("Navigation", "Toggle neighborhood mode")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "neighborhood_more",
            fixed(Action::ExpandNeighborhood),
            vec![chord('z', KeyDef::Char('+')), chord('z', KeyDef::Char('='))],
        )
        .help("Navigation", "Show more neighborhood")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "neighborhood_less",
            fixed(Action::ShrinkNeighborhood),
            vec![chord('z', KeyDef::Char('-'))],
        )
        .help("Navigation", "Show less neighborhood")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "set",
            fixed(Action::EnterMoveBookmarkMode),
            vec![chord('b', KeyDef::Char('m'))],
        )
        .help("Bookmarks & Git", "Set/move bookmark")
        .prefix_title("bookmark"),
        BindingSpec::new(
            Normal,
            "delete",
            fixed(Action::EnterBookmarkPicker(BookmarkSelectAction::Delete)),
            vec![chord('b', KeyDef::Char('d'))],
        )
        .help("Bookmarks & Git", "Delete bookmark")
        .prefix_title("bookmark"),
    ]
}
