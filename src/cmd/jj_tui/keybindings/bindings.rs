use super::ActionTemplate::*;
use super::BindingDef;
use super::KeyDef::*;
use super::ModeId::*;
use crate::cmd::jj_tui::action::Action;
use crate::cmd::jj_tui::state::{BookmarkSelectAction, RebaseType};
use ratatui::crossterm::event::KeyCode;

macro_rules! act {
    ($action:ident) => {
        Fixed(Action::$action)
    };
    ($action:ident($($arg:expr),*)) => {
        Fixed(Action::$action($($arg),*))
    };
}

pub(super) fn binding_defs() -> &'static [BindingDef] {
    BINDING_DEFS
}

pub(super) fn prefix_title(prefix: char) -> Option<&'static str> {
    PREFIX_TITLES
        .iter()
        .find(|(pending, _)| *pending == prefix)
        .map(|(_, title)| *title)
}

const PREFIX_TITLES: &[(char, &str)] = &[('g', "git"), ('z', "nav"), ('b', "bookmark")];

static BINDING_DEFS: &[BindingDef] = &[
    BindingDef::new(Normal, Char('q'), act!(Quit), "quit").help("General", "Quit"),
    BindingDef::new(Normal, Ctrl('c'), act!(Quit), "quit").alias(),
    BindingDef::new(Normal, Char('Q'), act!(EnterSquashMode), "squash")
        .help("Rebase", "Squash into target"),
    BindingDef::new(Normal, Key(KeyCode::Esc), NormalEscConditional, "esc"),
    BindingDef::new(Normal, Char('?'), act!(EnterHelp), "help").help("General", "Toggle help"),
    BindingDef::new(Normal, Char('j'), act!(MoveCursorDown), "down")
        .help("Navigation", "Move cursor down"),
    BindingDef::new(Normal, Key(KeyCode::Down), act!(MoveCursorDown), "down").alias(),
    BindingDef::new(Normal, Char('k'), act!(MoveCursorUp), "up")
        .help("Navigation", "Move cursor up"),
    BindingDef::new(Normal, Key(KeyCode::Up), act!(MoveCursorUp), "up").alias(),
    BindingDef::new(Normal, Char('@'), act!(JumpToWorkingCopy), "working-copy")
        .help("Navigation", "Jump to working copy"),
    BindingDef::new(Normal, Ctrl('u'), PageUpHalfViewport, "page-up").help("Navigation", "Page up"),
    BindingDef::new(Normal, Ctrl('d'), PageDownHalfViewport, "page-down")
        .help("Navigation", "Page down"),
    BindingDef::new(Normal, Char('g'), act!(SetPendingKey('g')), "git"),
    BindingDef::new(Normal, Char('z'), act!(SetPendingKey('z')), "nav"),
    BindingDef::new(Normal, Char('b'), act!(SetPendingKey('b')), "bookmark"),
    BindingDef::new(Normal, Char('f'), act!(ToggleFullMode), "full")
        .help("View", "Toggle full mode"),
    BindingDef::new(Normal, Key(KeyCode::Enter), act!(ToggleFocus), "zoom")
        .help("Navigation", "Zoom in/out on node"),
    BindingDef::new(Normal, Key(KeyCode::Tab), act!(ToggleExpanded), "details")
        .help("View", "Toggle commit details"),
    BindingDef::new(Normal, Char(' '), act!(ToggleExpanded), "details").alias(),
    BindingDef::new(Normal, Char('\\'), act!(ToggleSplitView), "split")
        .help("View", "Toggle split view"),
    BindingDef::new(Normal, Char('R'), act!(RefreshTree), "refresh").help("View", "Refresh tree"),
    BindingDef::new(Normal, Char('d'), act!(EnterDiffView), "diff").help("View", "View diff"),
    BindingDef::new(Normal, Char('D'), act!(EditDescription), "desc")
        .help("View", "Edit description"),
    BindingDef::new(Normal, Char('e'), act!(EditWorkingCopy), "edit")
        .help("Edit Operations", "Edit working copy (jj edit)"),
    BindingDef::new(Normal, Char('n'), act!(CreateNewCommit), "new")
        .help("Edit Operations", "New commit (jj new)"),
    BindingDef::new(Normal, Char('c'), act!(CommitWorkingCopy), "commit")
        .help("Edit Operations", "Commit changes (jj commit)"),
    BindingDef::new(Normal, Char('x'), act!(ToggleSelection), "toggle")
        .help("Selection", "Toggle selection"),
    BindingDef::new(Normal, Char('v'), act!(EnterSelecting), "select")
        .help("Selection", "Visual select mode"),
    BindingDef::new(Normal, Char('a'), act!(EnterConfirmAbandon), "abandon")
        .help("Selection", "Abandon selected"),
    BindingDef::new(
        Normal,
        Char('r'),
        act!(EnterRebaseMode(RebaseType::Single)),
        "rebase-single",
    )
    .help("Rebase", "Rebase single (-r)"),
    BindingDef::new(
        Normal,
        Char('s'),
        act!(EnterRebaseMode(RebaseType::WithDescendants)),
        "rebase-desc",
    )
    .help("Rebase", "Rebase + descendants (-s)"),
    BindingDef::new(
        Normal,
        Char('t'),
        act!(EnterConfirmRebaseOntoTrunk(RebaseType::Single)),
        "trunk-single",
    )
    .help("Rebase", "Quick rebase onto trunk"),
    BindingDef::new(
        Normal,
        Char('T'),
        act!(EnterConfirmRebaseOntoTrunk(RebaseType::WithDescendants)),
        "trunk-desc",
    )
    .help("Rebase", "Quick rebase tree onto trunk"),
    BindingDef::new(Normal, Char('u'), act!(Undo), "undo").help("Rebase", "Undo last operation"),
    BindingDef::new(Normal, Char('p'), act!(GitPush), "push")
        .help("Bookmarks & Git", "Push current bookmark"),
    BindingDef::new(Normal, Char('P'), act!(GitPushAll), "push-all"),
    BindingDef::new(Normal, Char('S'), act!(EnterConfirmStackSync), "stack-sync")
        .help("Bookmarks & Git", "Stack sync (fetch, rebase, clean up)"),
    BindingDef::new(Normal, Char('C'), act!(EnterConflicts), "conflicts")
        .help("Conflicts", "View conflicts panel"),
    BindingDef::new(Normal, Char('f'), act!(GitFetch), "fetch")
        .prefix('g')
        .help("Bookmarks & Git", "Git fetch"),
    BindingDef::new(Normal, Char('i'), act!(GitImport), "import")
        .prefix('g')
        .help("Bookmarks & Git", "Git import"),
    BindingDef::new(Normal, Char('e'), act!(GitExport), "export")
        .prefix('g')
        .help("Bookmarks & Git", "Git export"),
    BindingDef::new(
        Normal,
        Char('r'),
        act!(ResolveDivergence),
        "resolve-divergence",
    )
    .prefix('g')
    .help("Bookmarks & Git", "Resolve divergence (keep local)"),
    BindingDef::new(Normal, Char('p'), act!(CreatePR), "create-pr")
        .prefix('g')
        .help("Bookmarks & Git", "Create/open PR from bookmark"),
    BindingDef::new(Normal, Char('t'), act!(MoveCursorTop), "top")
        .prefix('z')
        .help("Navigation", "Jump to top"),
    BindingDef::new(Normal, Char('b'), act!(MoveCursorBottom), "bottom")
        .prefix('z')
        .help("Navigation", "Jump to bottom"),
    BindingDef::new(Normal, Char('z'), CenterCursorViewport, "center")
        .prefix('z')
        .help("Navigation", "Center current line"),
    BindingDef::new(Normal, Char('m'), act!(EnterMoveBookmarkMode), "set")
        .prefix('b')
        .help("Bookmarks & Git", "Set/move bookmark"),
    BindingDef::new(
        Normal,
        Char('d'),
        act!(EnterBookmarkPicker(BookmarkSelectAction::Delete)),
        "delete",
    )
    .prefix('b')
    .help("Bookmarks & Git", "Delete bookmark"),
    BindingDef::new(Help, Char('q'), act!(ExitHelp), "close"),
    BindingDef::new(Help, Char('?'), act!(ExitHelp), "close").alias(),
    BindingDef::new(Help, Key(KeyCode::Esc), act!(ExitHelp), "close").alias(),
    BindingDef::new(Help, Char('j'), act!(ScrollHelpDown(1)), "scroll-down"),
    BindingDef::new(
        Help,
        Key(KeyCode::Down),
        act!(ScrollHelpDown(1)),
        "scroll-down",
    )
    .alias(),
    BindingDef::new(Help, Char('k'), act!(ScrollHelpUp(1)), "scroll-up"),
    BindingDef::new(Help, Key(KeyCode::Up), act!(ScrollHelpUp(1)), "scroll-up").alias(),
    BindingDef::new(Help, Char('d'), act!(ScrollHelpDown(20)), "page-down"),
    BindingDef::new(Help, Char('u'), act!(ScrollHelpUp(20)), "page-up"),
    BindingDef::new(Diff, Char('j'), act!(ScrollDiffDown(1)), "scroll-down"),
    BindingDef::new(
        Diff,
        Key(KeyCode::Down),
        act!(ScrollDiffDown(1)),
        "scroll-down",
    )
    .alias(),
    BindingDef::new(Diff, Char('k'), act!(ScrollDiffUp(1)), "scroll-up"),
    BindingDef::new(Diff, Key(KeyCode::Up), act!(ScrollDiffUp(1)), "scroll-up").alias(),
    BindingDef::new(Diff, Char('d'), act!(ScrollDiffDown(20)), "page-down"),
    BindingDef::new(Diff, Char('u'), act!(ScrollDiffUp(20)), "page-up"),
    BindingDef::new(Diff, Char('z'), act!(SetPendingKey('z')), "nav"),
    BindingDef::new(Diff, Char('q'), act!(ExitDiffView), "close"),
    BindingDef::new(Diff, Key(KeyCode::Esc), act!(ExitDiffView), "close").alias(),
    BindingDef::new(Diff, Char('t'), act!(ScrollDiffTop), "top").prefix('z'),
    BindingDef::new(Diff, Char('b'), act!(ScrollDiffBottom), "bottom").prefix('z'),
    BindingDef::new(Confirm, Char('y'), act!(ConfirmYes), "yes"),
    BindingDef::new(Confirm, Key(KeyCode::Enter), act!(ConfirmYes), "yes").alias(),
    BindingDef::new(Confirm, Char('n'), act!(ConfirmNo), "no"),
    BindingDef::new(Confirm, Key(KeyCode::Esc), act!(ConfirmNo), "no").alias(),
    BindingDef::new(Selecting, Char('j'), act!(MoveCursorDown), "down"),
    BindingDef::new(Selecting, Key(KeyCode::Down), act!(MoveCursorDown), "down").alias(),
    BindingDef::new(Selecting, Char('k'), act!(MoveCursorUp), "up"),
    BindingDef::new(Selecting, Key(KeyCode::Up), act!(MoveCursorUp), "up").alias(),
    BindingDef::new(Selecting, Key(KeyCode::Esc), act!(ExitSelecting), "exit"),
    BindingDef::new(Selecting, Char('a'), act!(EnterConfirmAbandon), "abandon"),
    BindingDef::new(Rebase, Char('j'), act!(MoveRebaseDestDown), "dest-down"),
    BindingDef::new(
        Rebase,
        Key(KeyCode::Down),
        act!(MoveRebaseDestDown),
        "dest-down",
    )
    .alias(),
    BindingDef::new(Rebase, Char('k'), act!(MoveRebaseDestUp), "dest-up"),
    BindingDef::new(Rebase, Key(KeyCode::Up), act!(MoveRebaseDestUp), "dest-up").alias(),
    BindingDef::new(Rebase, Char('b'), act!(ToggleRebaseBranches), "branches"),
    BindingDef::new(Rebase, Key(KeyCode::Enter), act!(ExecuteRebase), "run"),
    BindingDef::new(Rebase, Key(KeyCode::Esc), act!(ExitRebaseMode), "cancel"),
    BindingDef::new(Squash, Char('j'), act!(MoveSquashDestDown), "dest-down"),
    BindingDef::new(
        Squash,
        Key(KeyCode::Down),
        act!(MoveSquashDestDown),
        "dest-down",
    )
    .alias(),
    BindingDef::new(Squash, Char('k'), act!(MoveSquashDestUp), "dest-up"),
    BindingDef::new(Squash, Key(KeyCode::Up), act!(MoveSquashDestUp), "dest-up").alias(),
    BindingDef::new(Squash, Key(KeyCode::Enter), act!(ExecuteSquash), "run"),
    BindingDef::new(Squash, Key(KeyCode::Esc), act!(ExitSquashMode), "cancel"),
    BindingDef::new(
        MovingBookmark,
        Char('j'),
        act!(MoveBookmarkDestDown),
        "dest-down",
    ),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Down),
        act!(MoveBookmarkDestDown),
        "dest-down",
    )
    .alias(),
    BindingDef::new(
        MovingBookmark,
        Char('k'),
        act!(MoveBookmarkDestUp),
        "dest-up",
    ),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Up),
        act!(MoveBookmarkDestUp),
        "dest-up",
    )
    .alias(),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Enter),
        act!(ExecuteBookmarkMove),
        "run",
    ),
    BindingDef::new(
        MovingBookmark,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    BindingDef::new(BookmarkSelect, Char('j'), act!(SelectBookmarkDown), "down"),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Down),
        act!(SelectBookmarkDown),
        "down",
    )
    .alias(),
    BindingDef::new(BookmarkSelect, Char('k'), act!(SelectBookmarkUp), "up"),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Up),
        act!(SelectBookmarkUp),
        "up",
    )
    .alias(),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Enter),
        act!(ConfirmBookmarkSelect),
        "select",
    ),
    BindingDef::new(
        BookmarkSelect,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Esc),
        act!(ExitBookmarkMode),
        "cancel",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Enter),
        act!(ConfirmBookmarkPicker),
        "confirm",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Down),
        act!(BookmarkPickerDown),
        "down",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Up),
        act!(BookmarkPickerUp),
        "up",
    ),
    BindingDef::new(
        BookmarkPicker,
        Key(KeyCode::Backspace),
        act!(BookmarkFilterBackspace),
        "backspace",
    )
    .alias(),
    BindingDef::new(BookmarkPicker, AnyChar, BookmarkFilterChar, "type"),
    BindingDef::new(
        PushSelect,
        Key(KeyCode::Esc),
        act!(ExitPushSelect),
        "cancel",
    ),
    BindingDef::new(
        PushSelect,
        Key(KeyCode::Enter),
        act!(PushSelectConfirm),
        "push",
    ),
    BindingDef::new(PushSelect, Key(KeyCode::Down), act!(PushSelectDown), "down"),
    BindingDef::new(PushSelect, Key(KeyCode::Up), act!(PushSelectUp), "up"),
    BindingDef::new(PushSelect, Char(' '), act!(PushSelectToggle), "toggle"),
    BindingDef::new(PushSelect, Char('a'), act!(PushSelectAll), "all"),
    BindingDef::new(PushSelect, Char('n'), act!(PushSelectNone), "none"),
    BindingDef::new(
        PushSelect,
        Key(KeyCode::Backspace),
        act!(PushSelectFilterBackspace),
        "backspace",
    )
    .alias(),
    BindingDef::new(PushSelect, AnyChar, PushSelectFilterChar, "type"),
    BindingDef::new(Conflicts, Char('j'), act!(ConflictsDown), "down"),
    BindingDef::new(Conflicts, Key(KeyCode::Down), act!(ConflictsDown), "down").alias(),
    BindingDef::new(Conflicts, Char('k'), act!(ConflictsUp), "up"),
    BindingDef::new(Conflicts, Key(KeyCode::Up), act!(ConflictsUp), "up").alias(),
    BindingDef::new(Conflicts, Key(KeyCode::Enter), act!(ConflictsJump), "jump"),
    BindingDef::new(
        Conflicts,
        Char('R'),
        act!(StartResolveFromConflicts),
        "resolve",
    ),
    BindingDef::new(Conflicts, Char('q'), act!(ExitConflicts), "exit"),
    BindingDef::new(Conflicts, Key(KeyCode::Esc), act!(ExitConflicts), "exit").alias(),
];
