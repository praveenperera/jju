# jju Development Guide

## jj_tui Architecture

### Data-First Architecture

The TUI follows a data-first architecture with clear separation of concerns:

```
KeyEvent → Controller → Action → Engine → Effect → Runner
```

- **Controller**: Maps `(ModeState, KeyEvent)` → `Action` (pure, no IO)
- **Engine**: Processes `reduce(state, Action)` → `Vec<Effect>` (pure, no IO)
- **Runner**: Executes `Effect` variants (performs IO via commands.rs)

### Module Structure

```
src/cmd/jj_tui/
├── app.rs           # App state, event loop (~280 lines)
├── action.rs        # Action enum (all user intents)
├── effect.rs        # Effect enum (all side effects)
├── engine.rs        # Pure reduce() function
├── runner.rs        # Effects executor (IO)
├── controller/      # Key → Action mapping
│   ├── mod.rs
│   ├── normal.rs
│   ├── diff.rs
│   ├── rebase.rs
│   ├── bookmark.rs
│   ├── squash.rs
│   ├── selection.rs
│   └── confirm.rs
├── state.rs         # ModeState enum, state types
├── commands.rs      # JJ command execution helpers
├── vm.rs            # TreeRowVm view model
├── ui.rs            # Rendering
├── tree.rs          # TreeState, node management
├── preview.rs       # Log parsing, NodeRole enum
├── theme.rs         # Color constants
├── keybindings.rs   # Keybinding definitions
└── handlers/
    └── diff.rs      # Diff parsing
```

### Key Patterns

**ModeState Enum**: Unifies mode and associated state into a single enum:
```rust
pub enum ModeState {
    Normal,
    Help,
    ViewingDiff(DiffState),
    Confirming(ConfirmState),
    Selecting,
    Rebasing(RebaseState),
    // ...
}
```

**Action Enum**: Represents all user intents:
```rust
pub enum Action {
    MoveCursorUp,
    EnterRebaseMode(RebaseType),
    ExecuteRebase,
    GitPush,
    Quit,
    // ...
}
```

**Effect Enum**: Represents all side effects:
```rust
pub enum Effect {
    RefreshTree,
    RunRebase { source, dest, rebase_type, allow_branches },
    SetStatus { text, kind },
    // ...
}
```

**TreeRowVm View Model**: Pre-computed view state separates computation from rendering:
```rust
pub struct TreeRowVm {
    pub is_cursor: bool,
    pub is_selected: bool,
    pub role: NodeRole,
    pub change_id_prefix: String,
    // ...
}
```

The data flow is: `App` → `vm::build_tree_view()` → `Vec<TreeRowVm>` → `ui::render_row()`

**Theme Constants**: All colors are defined in `theme.rs`:
```rust
pub const CURSOR_BG: Color = Color::Rgb(40, 40, 60);
pub const POPUP_BG: Color = Color::Rgb(20, 20, 30);
```

### Unicode Width

Use `unicode_width::UnicodeWidthStr` for layout calculations, not `.len()`:
```rust
use unicode_width::UnicodeWidthStr;
let width = text.width();  // correct for CJK, emoji
```

### Running Commands

JJ commands are executed via the runner, which calls `commands.rs`:
```rust
// commands.rs provides the low-level API
commands::revision::edit(&rev)?;
commands::git::push_bookmark(&name)?;
commands::rebase::single(&source, &dest)?;

// effects trigger these through the runner
Effect::RunEdit { rev }
Effect::RunGitPush { bookmark }
Effect::RunRebase { source, dest, rebase_type, allow_branches }
```

## Testing

```bash
cargo build
cargo clippy
cargo test
```

Manual testing should cover all modes: Normal, Help, Diff, Confirm, Select, Rebase, MovingBookmark, BookmarkInput, BookmarkSelect, BookmarkPicker, Squash.
