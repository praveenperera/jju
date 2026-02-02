# jju

A TUI for [Jujutsu](https://github.com/martinvonz/jj) version control.

> **Note:** Alpha software. I use the TUI daily but make changes constantly. Non-TUI commands might be broken and will probably be removed in the future.

<!-- TODO: Add terminal recording (asciinema/vhs) -->

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

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Select / Expand |
| `Esc` | Back / Cancel |
| `q` | Quit |

### Prefix Menus

| Key | Menu |
|-----|------|
| `g` | Go to (navigation) |
| `z` | Zoom controls |
| `b` | Bookmark actions |

### Actions

| Key | Action |
|-----|--------|
| `e` | Edit commit |
| `d` | Show diff |
| `r` | Rebase |
| `n` | New commit |
| `s` | Split |

## Building from Source

```bash
git clone https://github.com/praveenperera/jju
cd jju
cargo install --path .
```

## License

Apache-2.0
