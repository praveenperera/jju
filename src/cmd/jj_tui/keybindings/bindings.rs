use super::ActionTemplate::*;
use super::ModeId::*;
use super::{BindingBehavior, BindingSpec, KeyDef, KeySequence};
use crate::cmd::jj_tui::action::Action;
use crate::cmd::jj_tui::state::{BookmarkSelectAction, RebaseType};
use ratatui::crossterm::event::KeyCode;

macro_rules! act {
    ($action:ident) => {
        BindingBehavior::Action(Fixed(Action::$action))
    };
    ($action:ident($($arg:expr),*)) => {
        BindingBehavior::Action(Fixed(Action::$action($($arg),*)))
    };
}

macro_rules! single {
    ($key:expr) => {
        KeySequence::Single($key)
    };
}

macro_rules! chord {
    ($prefix:literal, $key:expr) => {
        KeySequence::Chord($prefix, $key)
    };
}

macro_rules! pending {
    ($title:literal) => {
        BindingBehavior::PendingPrefix { title: $title }
    };
}

pub(super) fn builtin_specs() -> Vec<BindingSpec> {
    vec![
        BindingSpec::new(
            Normal,
            "quit",
            act!(Quit),
            vec![single!(KeyDef::Char('q')), single!(KeyDef::Ctrl('c'))],
        )
        .help("General", "Quit"),
        BindingSpec::new(
            Normal,
            "squash",
            act!(EnterSquashMode),
            vec![single!(KeyDef::Char('Q'))],
        )
        .help("Rebase", "Squash into target"),
        BindingSpec::new(
            Normal,
            "esc",
            BindingBehavior::Action(NormalEscConditional),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            Normal,
            "help",
            act!(EnterHelp),
            vec![single!(KeyDef::Char('?'))],
        )
        .help("General", "Toggle help"),
        BindingSpec::new(
            Normal,
            "down",
            act!(MoveCursorDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        )
        .help("Navigation", "Move cursor down"),
        BindingSpec::new(
            Normal,
            "up",
            act!(MoveCursorUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        )
        .help("Navigation", "Move cursor up"),
        BindingSpec::new(
            Normal,
            "working-copy",
            act!(JumpToWorkingCopy),
            vec![single!(KeyDef::Char('@'))],
        )
        .help("Navigation", "Jump to working copy"),
        BindingSpec::new(
            Normal,
            "page-up",
            BindingBehavior::Action(PageUpHalfViewport),
            vec![single!(KeyDef::Ctrl('u'))],
        )
        .help("Navigation", "Page up"),
        BindingSpec::new(
            Normal,
            "page-down",
            BindingBehavior::Action(PageDownHalfViewport),
            vec![single!(KeyDef::Ctrl('d'))],
        )
        .help("Navigation", "Page down"),
        BindingSpec::new(
            Normal,
            "git",
            pending!("git"),
            vec![single!(KeyDef::Char('g'))],
        ),
        BindingSpec::new(
            Normal,
            "nav",
            pending!("nav"),
            vec![single!(KeyDef::Char('z'))],
        ),
        BindingSpec::new(
            Normal,
            "bookmark",
            pending!("bookmark"),
            vec![single!(KeyDef::Char('b'))],
        ),
        BindingSpec::new(
            Normal,
            "full",
            act!(ToggleFullMode),
            vec![single!(KeyDef::Char('f'))],
        )
        .help("View", "Toggle full mode"),
        BindingSpec::new(
            Normal,
            "zoom",
            act!(ToggleFocus),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        )
        .help("Navigation", "Zoom in/out on node"),
        BindingSpec::new(
            Normal,
            "details",
            act!(ToggleExpanded),
            vec![
                single!(KeyDef::Key(KeyCode::Tab)),
                single!(KeyDef::Char(' ')),
            ],
        )
        .help("View", "Toggle commit details"),
        BindingSpec::new(
            Normal,
            "split",
            act!(ToggleSplitView),
            vec![single!(KeyDef::Char('\\'))],
        )
        .help("View", "Toggle split view"),
        BindingSpec::new(
            Normal,
            "refresh",
            act!(RefreshTree),
            vec![single!(KeyDef::Char('R'))],
        )
        .help("View", "Refresh tree"),
        BindingSpec::new(
            Normal,
            "diff",
            act!(EnterDiffView),
            vec![single!(KeyDef::Char('d'))],
        )
        .help("View", "View diff"),
        BindingSpec::new(
            Normal,
            "desc",
            act!(EditDescription),
            vec![single!(KeyDef::Char('D'))],
        )
        .help("View", "Edit description"),
        BindingSpec::new(
            Normal,
            "edit",
            act!(EditWorkingCopy),
            vec![single!(KeyDef::Char('e'))],
        )
        .help("Edit Operations", "Edit working copy (jj edit)"),
        BindingSpec::new(
            Normal,
            "new",
            act!(CreateNewCommit),
            vec![single!(KeyDef::Char('n'))],
        )
        .help("Edit Operations", "New commit (jj new)"),
        BindingSpec::new(
            Normal,
            "commit",
            act!(CommitWorkingCopy),
            vec![single!(KeyDef::Char('c'))],
        )
        .help("Edit Operations", "Commit changes (jj commit)"),
        BindingSpec::new(
            Normal,
            "toggle",
            act!(ToggleSelection),
            vec![single!(KeyDef::Char('x'))],
        )
        .help("Selection", "Toggle selection"),
        BindingSpec::new(
            Normal,
            "select",
            act!(EnterSelecting),
            vec![single!(KeyDef::Char('v'))],
        )
        .help("Selection", "Visual select mode"),
        BindingSpec::new(
            Normal,
            "abandon",
            act!(EnterConfirmAbandon),
            vec![single!(KeyDef::Char('a'))],
        )
        .help("Selection", "Abandon selected"),
        BindingSpec::new(
            Normal,
            "rebase-single",
            act!(EnterRebaseMode(RebaseType::Single)),
            vec![single!(KeyDef::Char('r'))],
        )
        .help("Rebase", "Rebase single (-r)"),
        BindingSpec::new(
            Normal,
            "rebase-desc",
            act!(EnterRebaseMode(RebaseType::WithDescendants)),
            vec![single!(KeyDef::Char('s'))],
        )
        .help("Rebase", "Rebase + descendants (-s)"),
        BindingSpec::new(
            Normal,
            "trunk-single",
            act!(EnterConfirmRebaseOntoTrunk(RebaseType::Single)),
            vec![single!(KeyDef::Char('t'))],
        )
        .help("Rebase", "Quick rebase onto trunk"),
        BindingSpec::new(
            Normal,
            "trunk-desc",
            act!(EnterConfirmRebaseOntoTrunk(RebaseType::WithDescendants)),
            vec![single!(KeyDef::Char('T'))],
        )
        .help("Rebase", "Quick rebase tree onto trunk"),
        BindingSpec::new(Normal, "undo", act!(Undo), vec![single!(KeyDef::Char('u'))])
            .help("Rebase", "Undo last operation"),
        BindingSpec::new(
            Normal,
            "push",
            act!(GitPush),
            vec![single!(KeyDef::Char('p'))],
        )
        .help("Bookmarks & Git", "Push current bookmark"),
        BindingSpec::new(
            Normal,
            "push-all",
            act!(GitPushAll),
            vec![single!(KeyDef::Char('P'))],
        ),
        BindingSpec::new(
            Normal,
            "stack-sync",
            act!(EnterConfirmStackSync),
            vec![single!(KeyDef::Char('S'))],
        )
        .help("Bookmarks & Git", "Stack sync (fetch, rebase, clean up)"),
        BindingSpec::new(
            Normal,
            "conflicts",
            act!(EnterConflicts),
            vec![single!(KeyDef::Char('C'))],
        )
        .help("Conflicts", "View conflicts panel"),
        BindingSpec::new(
            Normal,
            "fetch",
            act!(GitFetch),
            vec![chord!('g', KeyDef::Char('f'))],
        )
        .help("Bookmarks & Git", "Git fetch")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "import",
            act!(GitImport),
            vec![chord!('g', KeyDef::Char('i'))],
        )
        .help("Bookmarks & Git", "Git import")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "export",
            act!(GitExport),
            vec![chord!('g', KeyDef::Char('e'))],
        )
        .help("Bookmarks & Git", "Git export")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "resolve-divergence",
            act!(ResolveDivergence),
            vec![chord!('g', KeyDef::Char('r'))],
        )
        .help("Bookmarks & Git", "Resolve divergence (keep local)")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "create-pr",
            act!(CreatePR),
            vec![chord!('g', KeyDef::Char('p'))],
        )
        .help("Bookmarks & Git", "Create/open PR from bookmark")
        .prefix_title("git"),
        BindingSpec::new(
            Normal,
            "top",
            act!(MoveCursorTop),
            vec![chord!('z', KeyDef::Char('t'))],
        )
        .help("Navigation", "Jump to top")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "bottom",
            act!(MoveCursorBottom),
            vec![chord!('z', KeyDef::Char('b'))],
        )
        .help("Navigation", "Jump to bottom")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "center",
            BindingBehavior::Action(CenterCursorViewport),
            vec![chord!('z', KeyDef::Char('z'))],
        )
        .help("Navigation", "Center current line")
        .prefix_title("nav"),
        BindingSpec::new(
            Normal,
            "set",
            act!(EnterMoveBookmarkMode),
            vec![chord!('b', KeyDef::Char('m'))],
        )
        .help("Bookmarks & Git", "Set/move bookmark")
        .prefix_title("bookmark"),
        BindingSpec::new(
            Normal,
            "delete",
            act!(EnterBookmarkPicker(BookmarkSelectAction::Delete)),
            vec![chord!('b', KeyDef::Char('d'))],
        )
        .help("Bookmarks & Git", "Delete bookmark")
        .prefix_title("bookmark"),
        BindingSpec::new(
            Help,
            "close",
            act!(ExitHelp),
            vec![
                single!(KeyDef::Char('q')),
                single!(KeyDef::Char('?')),
                single!(KeyDef::Key(KeyCode::Esc)),
            ],
        ),
        BindingSpec::new(
            Help,
            "scroll-down",
            act!(ScrollHelpDown(1)),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Help,
            "scroll-up",
            act!(ScrollHelpUp(1)),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            Help,
            "page-down",
            act!(ScrollHelpDown(20)),
            vec![single!(KeyDef::Char('d'))],
        ),
        BindingSpec::new(
            Help,
            "page-up",
            act!(ScrollHelpUp(20)),
            vec![single!(KeyDef::Char('u'))],
        ),
        BindingSpec::new(
            Diff,
            "scroll-down",
            act!(ScrollDiffDown(1)),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Diff,
            "scroll-up",
            act!(ScrollDiffUp(1)),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            Diff,
            "page-down",
            act!(ScrollDiffDown(20)),
            vec![single!(KeyDef::Char('d'))],
        ),
        BindingSpec::new(
            Diff,
            "page-up",
            act!(ScrollDiffUp(20)),
            vec![single!(KeyDef::Char('u'))],
        ),
        BindingSpec::new(
            Diff,
            "nav",
            pending!("nav"),
            vec![single!(KeyDef::Char('z'))],
        ),
        BindingSpec::new(
            Diff,
            "close",
            act!(ExitDiffView),
            vec![
                single!(KeyDef::Char('q')),
                single!(KeyDef::Key(KeyCode::Esc)),
            ],
        ),
        BindingSpec::new(
            Diff,
            "top",
            act!(ScrollDiffTop),
            vec![chord!('z', KeyDef::Char('t'))],
        )
        .prefix_title("nav"),
        BindingSpec::new(
            Diff,
            "bottom",
            act!(ScrollDiffBottom),
            vec![chord!('z', KeyDef::Char('b'))],
        )
        .prefix_title("nav"),
        BindingSpec::new(
            Confirm,
            "yes",
            act!(ConfirmYes),
            vec![
                single!(KeyDef::Char('y')),
                single!(KeyDef::Key(KeyCode::Enter)),
            ],
        ),
        BindingSpec::new(
            Confirm,
            "no",
            act!(ConfirmNo),
            vec![
                single!(KeyDef::Char('n')),
                single!(KeyDef::Key(KeyCode::Esc)),
            ],
        ),
        BindingSpec::new(
            Selecting,
            "down",
            act!(MoveCursorDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Selecting,
            "up",
            act!(MoveCursorUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            Selecting,
            "exit",
            act!(ExitSelecting),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            Selecting,
            "abandon",
            act!(EnterConfirmAbandon),
            vec![single!(KeyDef::Char('a'))],
        ),
        BindingSpec::new(
            Rebase,
            "dest-down",
            act!(MoveRebaseDestDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Rebase,
            "dest-up",
            act!(MoveRebaseDestUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            Rebase,
            "branches",
            act!(ToggleRebaseBranches),
            vec![single!(KeyDef::Char('b'))],
        ),
        BindingSpec::new(
            Rebase,
            "run",
            act!(ExecuteRebase),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            Rebase,
            "cancel",
            act!(ExitRebaseMode),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            Squash,
            "dest-down",
            act!(MoveSquashDestDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Squash,
            "dest-up",
            act!(MoveSquashDestUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            Squash,
            "run",
            act!(ExecuteSquash),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            Squash,
            "cancel",
            act!(ExitSquashMode),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            MovingBookmark,
            "dest-down",
            act!(MoveBookmarkDestDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            MovingBookmark,
            "dest-up",
            act!(MoveBookmarkDestUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            MovingBookmark,
            "run",
            act!(ExecuteBookmarkMove),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            MovingBookmark,
            "cancel",
            act!(ExitBookmarkMode),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            BookmarkSelect,
            "down",
            act!(SelectBookmarkDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            BookmarkSelect,
            "up",
            act!(SelectBookmarkUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            BookmarkSelect,
            "select",
            act!(ConfirmBookmarkSelect),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            BookmarkSelect,
            "cancel",
            act!(ExitBookmarkMode),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            BookmarkPicker,
            "cancel",
            act!(ExitBookmarkMode),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            BookmarkPicker,
            "confirm",
            act!(ConfirmBookmarkPicker),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            BookmarkPicker,
            "down",
            act!(BookmarkPickerDown),
            vec![single!(KeyDef::Key(KeyCode::Down))],
        ),
        BindingSpec::new(
            BookmarkPicker,
            "up",
            act!(BookmarkPickerUp),
            vec![single!(KeyDef::Key(KeyCode::Up))],
        ),
        BindingSpec::new(
            BookmarkPicker,
            "backspace",
            act!(BookmarkFilterBackspace),
            vec![single!(KeyDef::Key(KeyCode::Backspace))],
        ),
        BindingSpec::new(
            BookmarkPicker,
            "type",
            BindingBehavior::Action(BookmarkFilterChar),
            vec![single!(KeyDef::AnyChar)],
        ),
        BindingSpec::new(
            PushSelect,
            "cancel",
            act!(ExitPushSelect),
            vec![single!(KeyDef::Key(KeyCode::Esc))],
        ),
        BindingSpec::new(
            PushSelect,
            "push",
            act!(PushSelectConfirm),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            PushSelect,
            "down",
            act!(PushSelectDown),
            vec![single!(KeyDef::Key(KeyCode::Down))],
        ),
        BindingSpec::new(
            PushSelect,
            "up",
            act!(PushSelectUp),
            vec![single!(KeyDef::Key(KeyCode::Up))],
        ),
        BindingSpec::new(
            PushSelect,
            "toggle",
            act!(PushSelectToggle),
            vec![single!(KeyDef::Char(' '))],
        ),
        BindingSpec::new(
            PushSelect,
            "all",
            act!(PushSelectAll),
            vec![single!(KeyDef::Char('a'))],
        ),
        BindingSpec::new(
            PushSelect,
            "none",
            act!(PushSelectNone),
            vec![single!(KeyDef::Char('n'))],
        ),
        BindingSpec::new(
            PushSelect,
            "backspace",
            act!(PushSelectFilterBackspace),
            vec![single!(KeyDef::Key(KeyCode::Backspace))],
        ),
        BindingSpec::new(
            PushSelect,
            "type",
            BindingBehavior::Action(PushSelectFilterChar),
            vec![single!(KeyDef::AnyChar)],
        ),
        BindingSpec::new(
            Conflicts,
            "down",
            act!(ConflictsDown),
            vec![
                single!(KeyDef::Char('j')),
                single!(KeyDef::Key(KeyCode::Down)),
            ],
        ),
        BindingSpec::new(
            Conflicts,
            "up",
            act!(ConflictsUp),
            vec![
                single!(KeyDef::Char('k')),
                single!(KeyDef::Key(KeyCode::Up)),
            ],
        ),
        BindingSpec::new(
            Conflicts,
            "jump",
            act!(ConflictsJump),
            vec![single!(KeyDef::Key(KeyCode::Enter))],
        ),
        BindingSpec::new(
            Conflicts,
            "resolve",
            act!(StartResolveFromConflicts),
            vec![single!(KeyDef::Char('R'))],
        ),
        BindingSpec::new(
            Conflicts,
            "exit",
            act!(ExitConflicts),
            vec![
                single!(KeyDef::Char('q')),
                single!(KeyDef::Key(KeyCode::Esc)),
            ],
        ),
    ]
}
