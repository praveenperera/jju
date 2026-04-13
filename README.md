# jju

A TUI for [Jujutsu](https://github.com/martinvonz/jj) version control.

> **Alpha Software** — I use this TUI daily and it handles my regular workflow well. Expect rough edges and frequent changes.

[<video src="jju.mp4" autoplay loop muted playsinline></video>
](https://github.com/user-attachments/assets/71e79ae9-0725-4510-863e-236ffda48d25
)
## What Works Well

- **Rebasing** — visual rebase preview, single commit (`r`), with descendants (`s`), onto trunk (`t`/`T`)



- **Daily workflow** — editing, committing, diffing, bookmarks, undo
- **Navigation** — keyboard-driven with prefix menus and zoom levels

## Work in Progress

- Conflict resolution needs improvement

## Features

- **Commit tree visualization** with nested zoom levels
- **Visual rebase preview** before executing
- **Keyboard-driven navigation** with prefix menus (g/z/b)
- **Bookmark management** with picker and quick actions
- **Diff viewing** with syntax highlighting
- **Non-interactive hunk splitting**

## Installation

```bash
cargo install jju
```

### Requirements

- [Jujutsu](https://github.com/martinvonz/jj) installed and available in PATH

## Usage

```bash
jju
```

Start directly in neighborhood mode:

```bash
jju -n
# or
jju --neighborhood
```

Neighborhood mode stays anchored on the selected revision. Move the cursor freely, use `z+` / `z-` to zoom out toward the full tree or back into the anchored neighborhood, press `Enter` on a previewed branch to follow it, and use `Esc` to go back.

## Commands

| Command | Alias | Description |
| --- | --- | --- |
| `split-hunk` | `sh` | Split hunks from a commit non-interactively |
| `stack-sync` | `ss` | Sync the current stack with remote trunk |
| `tree` | `t` | Display the current stack as a tree |

## Keybindings

### Navigation

| Key                 | Action                          |
| ------------------- | ------------------------------- |
| `j` / `↓`           | Move down                       |
| `k` / `↑`           | Move up                         |
| `Enter`             | Toggle focus / open branch preview |
| `Tab` / `Space`     | Toggle expanded                 |
| `Esc`               | Back / cancel / clear selection |
| `@`                 | Jump to working copy            |
| `Ctrl+u` / `Ctrl+d` | Page up / down                  |
| `q`                 | Quit                            |
| `zn`                | Toggle neighborhood mode        |
| `z+` / `z-`         | Zoom neighborhood out / in      |

### Actions

| Key | Action              |
| --- | ------------------- |
| `e` | Edit commit         |
| `d` | Show diff           |
| `D` | Edit description    |
| `n` | New commit          |
| `c` | Commit working copy |
| `a` | Abandon commit      |
| `u` | Undo                |
| `Q` | Squash              |

### Rebase

| Key | Action                               |
| --- | ------------------------------------ |
| `r` | Rebase single commit                 |
| `s` | Rebase with descendants              |
| `t` | Rebase onto trunk (single)           |
| `T` | Rebase onto trunk (with descendants) |

### Selection

| Key | Action            |
| --- | ----------------- |
| `x` | Toggle selection  |
| `v` | Enter select mode |

### Git

| Key  | Action                |
| ---- | --------------------- |
| `p`  | Push current bookmark |
| `P`  | Push all bookmarks    |
| `gi` | Git import            |
| `ge` | Git export            |

### View

| Key | Action            |
| --- | ----------------- |
| `f` | Toggle full mode  |
| `\` | Toggle split view |
| `?` | Help              |

### Prefix Menus

| Key | Menu                                                   |
| --- | ------------------------------------------------------ |
| `g` | Git operations (`gi` import, `ge` export)              |
| `z` | Zoom/scroll (`zt` top, `zb` bottom, `zz` center)       |
| `b` | Bookmark actions (`bm` move, `bs` create, `bd` delete) |

### Custom Keybindings

Create a keybinding config at `$XDG_CONFIG_HOME/jju/keybindings.toml` or, if `XDG_CONFIG_HOME` is unset, `~/.config/jju/keybindings.toml`

The file format is:

```toml
version = 2

[[binding]]
mode = "normal"
command = "down"
keys = [["j"], ["Down"]]
```

- `mode`, `command`, and `keys` are required on each `[[binding]]`
- `keys` is a list of one-step or two-step key sequences
- supported tokens are single characters, `Ctrl+<char>`, `Enter`, `Esc`, `Tab`, `Backspace`, `Delete`/`Del`, `Up`, `Down`, `Left`, `Right`, `Space`, and `AnyChar`
- two-step sequences use arrays like `["g", "f"]`
- chord prefixes must be plain character keys, and `AnyChar` cannot be the second step of a chord
- overrides replace all built-in keys for that command in that mode, so repeat any defaults you want to keep
- a missing config file is ignored
- an invalid config falls back to the built-in keymap and shows a startup warning

Keep the default `down` bindings and add `J` as another alias:

```toml
version = 2

[[binding]]
mode = "normal"
command = "down"
keys = [["j"], ["J"], ["Down"]]
```

Move the git prefix from `g` to `X` by rebinding the prefix key and every command that uses it:

```toml
version = 2

[[binding]]
mode = "normal"
command = "git"
keys = [["X"]]

[[binding]]
mode = "normal"
command = "fetch"
keys = [["X", "f"]]

[[binding]]
mode = "normal"
command = "import"
keys = [["X", "i"]]

[[binding]]
mode = "normal"
command = "export"
keys = [["X", "e"]]

[[binding]]
mode = "normal"
command = "resolve_divergence"
keys = [["X", "r"]]

[[binding]]
mode = "normal"
command = "create_pr"
keys = [["X", "p"]]
```

See [`example-keybindings.toml`](./example-keybindings.toml) for a larger sample, including `AnyChar` bindings used by typed filter modes. The current command ids live in `src/cmd/jj_tui/keybindings/bindings.rs`

## Building from Source

```bash
git clone https://github.com/praveenperera/jju
cd jju
cargo install --path .
```

## License

Apache-2.0
