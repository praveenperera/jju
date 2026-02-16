# jju

A TUI for [Jujutsu](https://github.com/martinvonz/jj) version control.

> **Alpha Software** — I use this TUI daily and it handles my regular workflow well. Expect rough edges and frequent changes.

<video src="jju.mp4" autoplay loop muted playsinline></video>

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
| `Enter`             | Toggle focus / zoom             |
| `Tab` / `Space`     | Toggle expanded                 |
| `Esc`               | Back / Cancel / Clear selection |
| `@`                 | Jump to working copy            |
| `Ctrl+u` / `Ctrl+d` | Page up / down                  |
| `q`                 | Quit                            |

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

## Building from Source

```bash
git clone https://github.com/praveenperera/jju
cd jju
cargo install --path .
```

## License

Apache-2.0
