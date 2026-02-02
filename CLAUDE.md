# jju Development Guide

## jj_tui Architecture

### Module Structure

```
src/cmd/jj_tui/
├── app.rs        # App state, ModeState enum, event loop, key handlers
├── vm.rs         # TreeRowVm view model, build_tree_view()
├── ui.rs         # Rendering (converts vm to ratatui widgets)
├── tree.rs       # TreeState, node management, selection
├── preview.rs    # Log parsing, NodeRole enum
├── theme.rs      # Color constants (CURSOR_BG, POPUP_BG, etc.)
├── commands.rs   # JJ command execution helpers
└── keybindings.rs # Key handling, prefix menu definitions
```

### Key Patterns

**ModeState Enum**: Unifies mode and associated state into a single enum, eliminating invalid states:
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

**Theme Constants**: All colors are defined in `theme.rs` for consistency:
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

All JJ commands go through `commands.rs`. This centralizes command execution and makes the code cleaner:
```rust
use super::commands;
commands::revision::edit(&rev)?;
commands::git::push_bookmark(&name)?;
commands::rebase::single(&source, &dest)?;
commands::bookmark::delete(&name)?;
```

## Testing

```bash
cargo build
cargo clippy
cargo test
```

Manual testing should cover all modes: Normal, Help, Diff, Confirm, Select, Rebase, MovingBookmark, BookmarkInput, BookmarkSelect, BookmarkPicker, Squash.
