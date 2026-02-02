use super::commands;
use super::tree::TreeState;
use super::ui;
use crate::jj_lib_helpers::JjRepo;
use ahash::{HashSet, HashSetExt};
use eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::style::Color;
use ratatui::DefaultTerminal;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

pub struct KeyBinding {
    pub key: char,
    pub label: &'static str,
}

pub struct PrefixMenu {
    pub prefix: char,
    pub title: &'static str,
    pub bindings: &'static [KeyBinding],
}

pub const PREFIX_MENUS: &[PrefixMenu] = &[
    PrefixMenu {
        prefix: 'g',
        title: "git",
        bindings: &[
            KeyBinding { key: 'i', label: "import" },
            KeyBinding { key: 'e', label: "export" },
        ],
    },
    PrefixMenu {
        prefix: 'z',
        title: "nav",
        bindings: &[
            KeyBinding { key: 't', label: "top" },
            KeyBinding { key: 'b', label: "bottom" },
            KeyBinding { key: 'z', label: "center" },
        ],
    },
    PrefixMenu {
        prefix: 'b',
        title: "bookmark",
        bindings: &[
            KeyBinding { key: 'm', label: "move" },
            KeyBinding { key: 's', label: "set/new" },
            KeyBinding { key: 'd', label: "delete" },
        ],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    FileHeader,
    Hunk,
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub fg: Color,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub spans: Vec<StyledSpan>,
    pub kind: DiffLineKind,
}

#[derive(Debug, Clone)]
pub struct DiffState {
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub rev: String,
}

/// Unified mode state - single source of truth for current mode and its associated state
/// This replaces the old pattern of `Mode` enum + `Option<*State>` fields
#[derive(Debug, Clone)]
pub enum ModeState {
    Normal,
    Help,
    ViewingDiff(DiffState),
    Confirming(ConfirmState),
    Selecting,
    Rebasing(RebaseState),
    MovingBookmark(MovingBookmarkState),
    BookmarkInput(BookmarkInputState),
    BookmarkSelect(BookmarkSelectState),
    BookmarkPicker(BookmarkPickerState),
    Squashing(SquashState),
}

impl ModeState {
    /// Check if this is Help mode
    pub fn is_help(&self) -> bool {
        matches!(self, ModeState::Help)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseType {
    Single,          // -r: just this revision
    WithDescendants, // -s: revision + all descendants
}

impl std::fmt::Display for RebaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebaseType::Single => write!(f, "-r"),
            RebaseType::WithDescendants => write!(f, "-s"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    #[allow(dead_code)]
    Info,
    Success,
    Warning,
    Error,
}

pub struct StatusMessage {
    pub text: String,
    pub kind: MessageKind,
    pub expires: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Abandon,
    RebaseOntoTrunk(RebaseType),
    MoveBookmarkBackwards {
        bookmark_name: String,
        dest_rev: String,
        op_before: String,
    },
}

#[derive(Debug, Clone)]
pub struct ConfirmState {
    pub action: ConfirmAction,
    pub message: String,
    pub revs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RebaseState {
    pub source_rev: String,
    pub rebase_type: RebaseType,
    pub dest_cursor: usize,
    pub allow_branches: bool,
    pub op_before: String,
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone)]
pub struct MovingBookmarkState {
    pub bookmark_name: String,
    pub dest_cursor: usize,
    pub op_before: String,
}

#[derive(Debug, Clone)]
pub struct BookmarkInputState {
    pub name: String,
    pub cursor: usize,
    pub target_rev: String,
    pub deleting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkSelectAction {
    Move,
    Delete,
}

#[derive(Debug, Clone)]
pub struct BookmarkSelectState {
    pub bookmarks: Vec<String>,
    pub selected_index: usize,
    pub target_rev: String,
    pub action: BookmarkSelectAction,
}

/// State for picking a bookmark from all bookmarks with type-to-filter
#[derive(Debug, Clone)]
pub struct BookmarkPickerState {
    pub all_bookmarks: Vec<String>,
    pub filter: String,
    pub filter_cursor: usize,
    pub selected_index: usize,
    pub target_rev: String,
    pub action: BookmarkSelectAction,
}

impl BookmarkPickerState {
    /// Get bookmarks that match the current filter
    pub fn filtered_bookmarks(&self) -> Vec<&String> {
        if self.filter.is_empty() {
            self.all_bookmarks.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.all_bookmarks
                .iter()
                .filter(|b| b.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SquashState {
    pub source_rev: String,
    pub dest_cursor: usize,
    pub op_before: String,
}

pub struct PendingSquash {
    pub source_rev: String,
    pub target_rev: String,
    pub op_before: String,
}

pub struct App {
    pub tree: TreeState,
    pub mode: ModeState,
    pub should_quit: bool,
    pub split_view: bool,
    pub diff_stats_cache: std::collections::HashMap<String, DiffStats>,
    pub status_message: Option<StatusMessage>,
    pub pending_editor: Option<String>,
    pub pending_squash: Option<PendingSquash>,
    pub last_op: Option<String>,
    pub pending_key: Option<char>,
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) theme_set: ThemeSet,
}

impl App {
    pub fn new() -> Result<Self> {
        let jj_repo = JjRepo::load(None)?;
        let tree = TreeState::load(&jj_repo)?;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Ok(Self {
            tree,
            mode: ModeState::Normal,
            should_quit: false,
            split_view: false,
            diff_stats_cache: std::collections::HashMap::new(),
            status_message: None,
            pending_editor: None,
            pending_squash: None,
            last_op: None,
            pending_key: None,
            syntax_set,
            theme_set,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let result = self.run_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            // handle pending editor launch
            if let Some(rev) = self.pending_editor.take() {
                ratatui::restore();
                let status = std::process::Command::new("jj")
                    .args(["describe", "-r", &rev])
                    .status();
                *terminal = ratatui::init();

                match status {
                    Ok(s) if s.success() => {
                        self.set_status("Description updated", MessageKind::Success);
                        let _ = self.refresh_tree();
                    }
                    Ok(_) => self.set_status("Editor cancelled", MessageKind::Warning),
                    Err(e) => self
                        .set_status(&format!("Failed to launch editor: {e}"), MessageKind::Error),
                }
                continue;
            }

            // handle pending squash (may open editor for combined description)
            if let Some(squash) = self.pending_squash.take() {
                ratatui::restore();
                let status = std::process::Command::new("jj")
                    .args(["squash", "-f", &squash.source_rev, "-t", &squash.target_rev])
                    .status();
                *terminal = ratatui::init();

                match status {
                    Ok(s) if s.success() => {
                        self.last_op = Some(squash.op_before);
                        let has_conflicts = self.check_conflicts();
                        let _ = self.refresh_tree();

                        if has_conflicts {
                            self.set_status(
                                "Squash created conflicts. Press u to undo",
                                MessageKind::Warning,
                            );
                        } else {
                            self.set_status("Squash complete", MessageKind::Success);
                        }
                    }
                    Ok(_) => self.set_status("Squash cancelled", MessageKind::Warning),
                    Err(e) => {
                        self.set_status(&format!("Squash failed: {e}"), MessageKind::Error)
                    }
                }
                continue;
            }

            let viewport_height = terminal.size()?.height.saturating_sub(3) as usize;
            self.tree.update_scroll(viewport_height);

            // fetch diff stats for expanded entry if needed
            self.ensure_expanded_stats();

            terminal.draw(|frame| ui::render(frame, self))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key, viewport_height);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent, viewport_height: usize) {
        // clear expired status messages
        if let Some(ref msg) = self.status_message {
            if Instant::now() > msg.expires {
                self.status_message = None;
            }
        }

        match &self.mode {
            ModeState::Normal => self.handle_normal_key(key, viewport_height),
            ModeState::Help => self.handle_help_key(key.code),
            ModeState::ViewingDiff(_) => self.handle_diff_key(key),
            ModeState::Confirming(_) => self.handle_confirm_key(key.code),
            ModeState::Selecting => self.handle_selecting_key(key, viewport_height),
            ModeState::Rebasing(_) => self.handle_rebasing_key(key.code),
            ModeState::MovingBookmark(_) => self.handle_moving_bookmark_key(key.code),
            ModeState::BookmarkInput(_) => self.handle_bookmark_input_key(key),
            ModeState::BookmarkSelect(_) => self.handle_bookmark_select_key(key.code),
            ModeState::BookmarkPicker(_) => self.handle_bookmark_picker_key(key),
            ModeState::Squashing(_) => self.handle_squashing_key(key.code),
        }
    }

    fn handle_normal_key(&mut self, key: event::KeyEvent, viewport_height: usize) {
        let code = key.code;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // handle pending key sequences
        if let Some(pending) = self.pending_key.take() {
            match (pending, code) {
                // 'g' prefix - git operations
                ('g', KeyCode::Char('i')) => {
                    let _ = self.git_import();
                }
                ('g', KeyCode::Char('e')) => {
                    let _ = self.git_export();
                }
                // 'z' prefix - navigation
                ('z', KeyCode::Char('t')) => self.tree.move_cursor_top(),
                ('z', KeyCode::Char('b')) => self.tree.move_cursor_bottom(),
                ('z', KeyCode::Char('z')) => self.center_cursor_in_view(viewport_height),
                // 'b' prefix - bookmark operations
                ('b', KeyCode::Char('m')) => {
                    let _ = self.enter_move_bookmark_mode();
                }
                ('b', KeyCode::Char('s')) => {
                    let _ = self.enter_create_bookmark();
                }
                ('b', KeyCode::Char('d')) => {
                    let _ = self.delete_bookmark_with_picker();
                }
                // any other key after prefix - ignore
                _ => {}
            }
            return;
        }

        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('Q') => {
                let _ = self.enter_squash_mode();
            }
            KeyCode::Esc => {
                if self.tree.is_focused() {
                    self.tree.unfocus();
                } else if !self.tree.selected.is_empty() {
                    self.tree.clear_selection();
                }
            }
            KeyCode::Char('?') => self.mode = ModeState::Help,

            KeyCode::Char('j') | KeyCode::Down => self.tree.move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => self.tree.move_cursor_up(),
            KeyCode::Char('@') => self.tree.jump_to_working_copy(),

            // multi-key sequence prefixes
            KeyCode::Char('g') => self.pending_key = Some('g'),
            KeyCode::Char('z') => self.pending_key = Some('z'),
            KeyCode::Char('b') => self.pending_key = Some('b'),

            KeyCode::Char('f') => self.tree.toggle_full_mode(),

            // diff viewing
            KeyCode::Char('D') => {
                let _ = self.enter_diff_view();
            }

            // zoom in/out on node
            KeyCode::Enter => self.tree.toggle_focus(),

            // details toggle
            KeyCode::Tab | KeyCode::Char(' ') => self.tree.toggle_expanded(),

            // page scrolling
            KeyCode::Char('u') if ctrl => self.tree.page_up(viewport_height / 2),
            KeyCode::Char('d') if ctrl => self.tree.page_down(viewport_height / 2),

            // split view toggle
            KeyCode::Char('\\') => self.split_view = !self.split_view,

            // edit operations
            KeyCode::Char('d') => self.enter_edit_description(),
            KeyCode::Char('e') => {
                let _ = self.edit_working_copy();
            }
            KeyCode::Char('n') => {
                let _ = self.create_new_commit();
            }
            KeyCode::Char('c') if ctrl => self.should_quit = true,
            KeyCode::Char('c') => {
                let _ = self.commit_working_copy();
            }

            // selection
            KeyCode::Char('x') => self.toggle_selection(),
            KeyCode::Char('v') => self.enter_visual_selection(),
            KeyCode::Char('a') => self.request_abandon(),

            // rebase operations
            KeyCode::Char('r') => {
                let _ = self.enter_rebase_mode(RebaseType::Single);
            }
            KeyCode::Char('s') => {
                let _ = self.enter_rebase_mode(RebaseType::WithDescendants);
            }
            KeyCode::Char('t') => {
                let _ = self.quick_rebase_onto_trunk(RebaseType::Single);
            }
            KeyCode::Char('T') => {
                let _ = self.quick_rebase_onto_trunk(RebaseType::WithDescendants);
            }

            // undo
            KeyCode::Char('u') => {
                let _ = self.undo_last_operation();
            }

            // git push
            KeyCode::Char('p') => {
                let _ = self.git_push();
            }

            _ => {}
        }
    }

    fn center_cursor_in_view(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        let half = viewport_height / 2;
        self.tree.scroll_offset = self.tree.cursor.saturating_sub(half);
    }

    fn git_push(&mut self) -> Result<()> {
        let Some(node) = self.tree.current_node() else {
            self.set_status("No revision selected", MessageKind::Error);
            return Ok(());
        };

        if node.bookmarks.is_empty() {
            self.set_status("No bookmark on this revision to push", MessageKind::Warning);
            return Ok(());
        }

        // push all bookmarks on this revision
        let bookmark_names = node.bookmark_names();
        let name = &bookmark_names[0];
        match commands::git::push_bookmark(name) {
            Ok(_) => {
                let _ = self.refresh_tree();
                self.set_status(&format!("Pushed bookmark '{name}'"), MessageKind::Success);
            }
            Err(e) => {
                self.set_status(&format!("Push failed: {e}"), MessageKind::Error);
            }
        }
        Ok(())
    }

    fn git_import(&mut self) -> Result<()> {
        match commands::git::import() {
            Ok(_) => {
                let _ = self.refresh_tree();
                self.set_status("Git import complete", MessageKind::Success);
            }
            Err(e) => {
                self.set_status(&format!("Git import failed: {e}"), MessageKind::Error);
            }
        }
        Ok(())
    }

    fn git_export(&mut self) -> Result<()> {
        match commands::git::export() {
            Ok(_) => {
                let _ = self.refresh_tree();
                self.set_status("Git export complete", MessageKind::Success);
            }
            Err(e) => {
                self.set_status(&format!("Git export failed: {e}"), MessageKind::Error);
            }
        }
        Ok(())
    }

    fn handle_help_key(&mut self, code: KeyCode) {
        if matches!(code, KeyCode::Char('q' | '?') | KeyCode::Esc) {
            self.mode = ModeState::Normal;
        }
    }

    fn handle_diff_key(&mut self, key: event::KeyEvent) {
        let code = key.code;

        // handle pending key sequences in diff view
        if let Some(pending) = self.pending_key.take() {
            if let ModeState::ViewingDiff(ref mut state) = self.mode {
                match (pending, code) {
                    ('z', KeyCode::Char('t')) => state.scroll_offset = 0,
                    ('z', KeyCode::Char('b')) => {
                        state.scroll_offset = state.lines.len().saturating_sub(1)
                    }
                    _ => {}
                }
            }
            return;
        }

        if let ModeState::ViewingDiff(ref mut state) = self.mode {
            match code {
                KeyCode::Char('j') | KeyCode::Down => {
                    state.scroll_offset = state.scroll_offset.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    state.scroll_offset = state.scroll_offset.saturating_sub(1);
                }
                KeyCode::Char('d') => {
                    state.scroll_offset = state.scroll_offset.saturating_add(20);
                }
                KeyCode::Char('u') => {
                    state.scroll_offset = state.scroll_offset.saturating_sub(20);
                }
                KeyCode::Char('z') => {
                    self.pending_key = Some('z');
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = ModeState::Normal;
                }
                _ => {}
            }
        } else {
            // no diff state, return to normal
            self.mode = ModeState::Normal;
        }
    }

    fn enter_diff_view(&mut self) -> Result<()> {
        let rev = self.current_rev();
        let diff_output = commands::diff::get_diff(&rev)?;
        let lines = parse_diff(&diff_output, &self.syntax_set, &self.theme_set);
        self.mode = ModeState::ViewingDiff(DiffState {
            lines,
            scroll_offset: 0,
            rev: rev.to_string(),
        });
        Ok(())
    }

    fn current_rev(&self) -> String {
        self.tree
            .current_node()
            .map(|n| n.change_id.clone())
            .unwrap_or_default()
    }

    pub fn get_diff_stats(&mut self, change_id: &str) -> Option<&DiffStats> {
        if !self.diff_stats_cache.contains_key(change_id) {
            if let Ok(stats) = self.fetch_diff_stats(change_id) {
                self.diff_stats_cache.insert(change_id.to_string(), stats);
            }
        }
        self.diff_stats_cache.get(change_id)
    }

    fn fetch_diff_stats(&self, change_id: &str) -> Result<DiffStats> {
        let output = commands::diff::get_stats(change_id)?;

        // parse output like: "3 files changed, 45 insertions(+), 12 deletions(-)"
        // or individual file lines and final summary
        let mut files_changed = 0;
        let mut insertions = 0;
        let mut deletions = 0;

        for line in output.lines() {
            // look for the summary line
            if line.contains("file") && line.contains("changed") {
                // parse: "N file(s) changed, M insertion(s)(+), K deletion(s)(-)"
                for part in line.split(',') {
                    let part = part.trim();
                    if part.contains("file") {
                        if let Some(num) = part.split_whitespace().next() {
                            files_changed = num.parse().unwrap_or(0);
                        }
                    } else if part.contains("insertion") {
                        if let Some(num) = part.split_whitespace().next() {
                            insertions = num.parse().unwrap_or(0);
                        }
                    } else if part.contains("deletion") {
                        if let Some(num) = part.split_whitespace().next() {
                            deletions = num.parse().unwrap_or(0);
                        }
                    }
                }
            }
        }

        Ok(DiffStats {
            files_changed,
            insertions,
            deletions,
        })
    }

    pub fn ensure_expanded_stats(&mut self) {
        if let Some(entry) = self.tree.current_entry() {
            if self.tree.is_expanded(self.tree.cursor) {
                let node = &self.tree.nodes[entry.node_index];
                let change_id = node.change_id.clone();
                let _ = self.get_diff_stats(&change_id);
            }
        }
    }

    fn set_status(&mut self, text: &str, kind: MessageKind) {
        self.status_message = Some(StatusMessage {
            text: text.to_string(),
            kind,
            expires: Instant::now() + Duration::from_secs(3),
        });
    }

    /// Save error details to a temp file and return the path
    fn save_error_to_file(error: &str) -> Option<String> {
        let temp_dir = std::env::temp_dir();
        let error_file = temp_dir.join(format!("jju-error-{}.log", std::process::id()));
        let path = error_file.to_string_lossy().to_string();

        match fs::File::create(&error_file) {
            Ok(mut file) => {
                if file.write_all(error.as_bytes()).is_ok() {
                    Some(path)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Set error status with details saved to file
    fn set_error_with_details(&mut self, prefix: &str, stderr: &str) {
        let first_line = stderr.lines().next().unwrap_or(stderr);
        let truncated = if first_line.len() > 80 {
            format!("{}...", &first_line[..77])
        } else {
            first_line.to_string()
        };

        if let Some(path) = Self::save_error_to_file(stderr) {
            self.set_status(
                &format!("{prefix}: {truncated} (full error: {path})"),
                MessageKind::Error,
            );
        } else {
            self.set_status(&format!("{prefix}: {truncated}"), MessageKind::Error);
        }
    }

    /// Check if moving a bookmark from current position to dest would be moving backwards
    /// Returns true if dest is an ancestor of the bookmark's current position
    fn is_bookmark_move_backwards(&self, bookmark_name: &str, dest_rev: &str) -> bool {
        let Some(current_node) = self.tree.nodes.iter().find(|n| n.has_bookmark(bookmark_name))
        else {
            return false; // new bookmark, not backwards
        };
        let current_change_id = &current_node.change_id;

        // Check if dest is an ancestor of current position
        commands::is_ancestor(dest_rev, current_change_id).unwrap_or(false)
    }

    fn refresh_tree(&mut self) -> Result<()> {
        // save current position to restore after refresh
        let current_change_id = self.tree.current_node().map(|n| n.change_id.clone());
        // save focus stack change_ids to restore after refresh
        let focus_stack_change_ids: Vec<String> = self
            .tree
            .focus_stack
            .iter()
            .filter_map(|&idx| self.tree.nodes.get(idx).map(|n| n.change_id.clone()))
            .collect();

        let jj_repo = JjRepo::load(None)?;
        self.tree = TreeState::load(&jj_repo)?;
        self.tree.clear_selection();
        self.diff_stats_cache.clear();

        // restore focus stack if the focused nodes still exist
        for change_id in focus_stack_change_ids {
            if let Some(node_idx) = self
                .tree
                .nodes
                .iter()
                .position(|n| n.change_id == change_id)
            {
                self.tree.focus_on(node_idx);
            }
        }

        // restore cursor to same change_id if it still exists
        if let Some(change_id) = current_change_id {
            if let Some(idx) = self
                .tree
                .visible_entries
                .iter()
                .position(|e| self.tree.nodes[e.node_index].change_id == change_id)
            {
                self.tree.cursor = idx;
            }
        }

        Ok(())
    }

    // Edit operations

    fn edit_working_copy(&mut self) -> Result<()> {
        let rev = self.current_rev();
        if let Some(node) = self.tree.current_node() {
            if node.is_working_copy {
                self.set_status("Already editing this revision", MessageKind::Warning);
                return Ok(());
            }
        }
        match commands::revision::edit(&rev) {
            Ok(_) => {
                self.set_status(&format!("Now editing {rev}"), MessageKind::Success);
                self.refresh_tree()?;
            }
            Err(e) => self.set_status(&format!("Edit failed: {e}"), MessageKind::Error),
        }
        Ok(())
    }

    fn create_new_commit(&mut self) -> Result<()> {
        let rev = self.current_rev();
        match commands::revision::new(&rev) {
            Ok(_) => {
                self.set_status("Created new commit", MessageKind::Success);
                self.refresh_tree()?;
                self.tree.jump_to_working_copy();
            }
            Err(e) => self.set_status(&format!("Failed: {e}"), MessageKind::Error),
        }
        Ok(())
    }

    fn commit_working_copy(&mut self) -> Result<()> {
        if let Some(node) = self.tree.current_node() {
            if !node.is_working_copy {
                self.set_status(
                    "Can only commit from working copy (@)",
                    MessageKind::Warning,
                );
                return Ok(());
            }
        }
        // use -m with current description to avoid opening $EDITOR
        let desc = self
            .tree
            .current_node()
            .map(|n| n.description.clone())
            .unwrap_or_default();
        let desc = if desc.is_empty() {
            "(no description)".to_string()
        } else {
            desc
        };
        match commands::revision::commit(&desc) {
            Ok(_) => {
                self.set_status("Changes committed", MessageKind::Success);
                self.refresh_tree()?;
            }
            Err(e) => self.set_status(&format!("Commit failed: {e}"), MessageKind::Error),
        }
        Ok(())
    }

    // Selection operations

    fn toggle_selection(&mut self) {
        self.tree.toggle_selected(self.tree.cursor);
    }

    fn enter_visual_selection(&mut self) {
        self.tree.selection_anchor = Some(self.tree.cursor);
        self.tree.selected.insert(self.tree.cursor);
        self.mode = ModeState::Selecting;
    }

    fn handle_selecting_key(&mut self, key: event::KeyEvent, _viewport_height: usize) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.tree.move_cursor_down();
                self.extend_selection_to_cursor();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.tree.move_cursor_up();
                self.extend_selection_to_cursor();
            }
            KeyCode::Esc => {
                self.mode = ModeState::Normal;
                self.tree.selection_anchor = None;
            }
            KeyCode::Char('a') => self.request_abandon(),
            _ => {}
        }
    }

    fn extend_selection_to_cursor(&mut self) {
        if let Some(anchor) = self.tree.selection_anchor {
            self.tree.selected.clear();
            self.tree.select_range(anchor, self.tree.cursor);
        }
    }

    // Confirmation dialog

    fn request_abandon(&mut self) {
        let revs: Vec<String> = if self.tree.selected.is_empty() {
            vec![self.current_rev()]
        } else {
            self.tree
                .selected
                .iter()
                .filter_map(|&idx| {
                    self.tree
                        .visible_entries
                        .get(idx)
                        .map(|e| self.tree.nodes[e.node_index].change_id.clone())
                })
                .collect()
        };

        // check for working copy in selection
        for rev in &revs {
            if self
                .tree
                .nodes
                .iter()
                .any(|n| n.change_id == *rev && n.is_working_copy)
            {
                self.set_status("Cannot abandon working copy", MessageKind::Error);
                return;
            }
        }

        let count = revs.len();
        let message = if count == 1 {
            format!("Abandon revision {}?", revs[0])
        } else {
            format!("Abandon {} revisions?", count)
        };

        self.mode = ModeState::Confirming(ConfirmState {
            action: ConfirmAction::Abandon,
            message,
            revs,
        });
    }

    fn handle_confirm_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Enter => self.execute_confirmed_action(),
            KeyCode::Char('n') | KeyCode::Esc => self.cancel_confirmation(),
            _ => {}
        }
    }

    fn execute_confirmed_action(&mut self) {
        let ModeState::Confirming(state) = std::mem::replace(&mut self.mode, ModeState::Normal)
        else {
            return;
        };
            match state.action {
                ConfirmAction::Abandon => {
                    let revset = state.revs.join(" | ");
                    match commands::revision::abandon(&revset) {
                        Ok(_) => {
                            let count = state.revs.len();
                            let msg = if count == 1 {
                                "Revision abandoned".to_string()
                            } else {
                                format!("{} revisions abandoned", count)
                            };
                            self.set_status(&msg, MessageKind::Success);
                            let _ = self.refresh_tree();
                        }
                        Err(e) => {
                            let error_details = format!("{e}");
                            self.set_error_with_details("Abandon failed", &error_details);
                        }
                    }
                }
                ConfirmAction::RebaseOntoTrunk(rebase_type) => {
                    let source = self.current_rev();
                    let op_before = self.get_current_operation_id().unwrap_or_default();

                    let result = match rebase_type {
                        RebaseType::Single => commands::rebase::single_onto_trunk(&source),
                        RebaseType::WithDescendants => {
                            commands::rebase::with_descendants_onto_trunk(&source)
                        }
                    };

                    match result {
                        Ok(_) => {
                            self.last_op = Some(op_before);
                            let has_conflicts = self.check_conflicts();
                            let _ = self.refresh_tree();

                            if has_conflicts {
                                self.set_status(
                                    "Rebased onto trunk (conflicts detected, u to undo)",
                                    MessageKind::Warning,
                                );
                            } else {
                                self.set_status("Rebased onto trunk", MessageKind::Success);
                            }
                        }
                        Err(e) => {
                            let error_details = format!("{e}");
                            self.set_error_with_details("Rebase failed", &error_details);
                        }
                    }
                }
                ConfirmAction::MoveBookmarkBackwards {
                    bookmark_name,
                    dest_rev,
                    op_before,
                } => {
                    self.do_bookmark_move(&bookmark_name, &dest_rev, &op_before, true);
                }
            }
            self.tree.clear_selection();
        // mode is already set to Normal by the mem::replace above
    }

    fn cancel_confirmation(&mut self) {
        self.mode = ModeState::Normal;
    }

    // Description editing

    fn enter_edit_description(&mut self) {
        self.pending_editor = Some(self.current_rev());
    }

    // Rebase operations

    fn get_current_operation_id(&self) -> Result<String> {
        commands::get_current_op_id()
    }

    fn enter_rebase_mode(&mut self, rebase_type: RebaseType) -> Result<()> {
        let source_rev = self.current_rev();
        if source_rev.is_empty() {
            self.set_status("No revision selected", MessageKind::Error);
            return Ok(());
        }

        // capture current operation ID for potential undo
        let op_before = self.get_current_operation_id().unwrap_or_default();
        let current = self.tree.cursor;

        // create initial state with cursor at current position
        let mut state = RebaseState {
            source_rev: source_rev.clone(),
            rebase_type,
            dest_cursor: current,
            allow_branches: false,
            op_before,
        };

        // temporarily set mode to compute moving indices
        self.mode = ModeState::Rebasing(state.clone());

        // find source's parent so initial preview shows source at its original position
        let moving = self.compute_moving_indices();
        let max = self.tree.visible_count();

        // get source's structural depth
        let source_struct_depth = self
            .tree
            .visible_entries
            .get(current)
            .map(|e| self.tree.nodes[e.node_index].depth)
            .unwrap_or(0);

        // find source's parent: closest entry above with smaller structural depth
        let mut initial_cursor = current.saturating_sub(1);
        while initial_cursor > 0 {
            let entry = &self.tree.visible_entries[initial_cursor];
            let node = &self.tree.nodes[entry.node_index];
            if node.depth < source_struct_depth && !moving.contains(&initial_cursor) {
                break;
            }
            initial_cursor -= 1;
        }

        // verify we found a valid non-moving entry
        if moving.contains(&initial_cursor) || initial_cursor >= max {
            // fallback: search forward for any non-moving entry
            initial_cursor = 0;
            while initial_cursor < max && moving.contains(&initial_cursor) {
                initial_cursor += 1;
            }
        }

        state.dest_cursor = initial_cursor;
        self.mode = ModeState::Rebasing(state);
        Ok(())
    }

    fn handle_rebasing_key(&mut self, code: KeyCode) {
        let ModeState::Rebasing(ref state) = self.mode else {
            self.mode = ModeState::Normal;
            return;
        };
        let state = state.clone();

        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_rebase_dest_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_rebase_dest_up();
            }
            KeyCode::Char('b') => {
                if let ModeState::Rebasing(ref mut s) = self.mode {
                    s.allow_branches = !s.allow_branches;
                }
            }
            KeyCode::Enter => {
                let _ = self.execute_rebase(&state);
            }
            KeyCode::Esc => {
                self.cancel_rebase();
            }
            _ => {}
        }
    }

    fn move_rebase_dest_up(&mut self) {
        let moving = self.compute_moving_indices();
        if let ModeState::Rebasing(ref mut state) = self.mode {
            let mut next = state.dest_cursor.saturating_sub(1);
            // skip over moving entries
            while next > 0 && moving.contains(&next) {
                next -= 1;
            }
            // only move if we found a valid non-moving position
            if !moving.contains(&next) {
                state.dest_cursor = next;
            }
        }
    }

    fn move_rebase_dest_down(&mut self) {
        let moving = self.compute_moving_indices();
        let max = self.tree.visible_count();
        if let ModeState::Rebasing(ref mut state) = self.mode {
            let mut next = state.dest_cursor + 1;
            // skip over moving entries
            while next < max && moving.contains(&next) {
                next += 1;
            }
            // only move if we found a valid position
            if next < max {
                state.dest_cursor = next;
            }
        }
    }

    fn get_rev_at_cursor(&self, cursor: usize) -> Option<String> {
        self.tree
            .visible_entries
            .get(cursor)
            .map(|e| self.tree.nodes[e.node_index].change_id.clone())
    }

    fn get_first_child(&self, rev: &str) -> Result<Option<String>> {
        commands::get_first_child(rev)
    }

    fn execute_rebase(&mut self, state: &RebaseState) -> Result<()> {
        let source = &state.source_rev;
        let Some(dest) = self.get_rev_at_cursor(state.dest_cursor) else {
            self.set_status("Invalid destination", MessageKind::Error);
            return Ok(());
        };

        // don't allow rebasing onto self
        if *source == dest {
            self.set_status("Cannot rebase onto self", MessageKind::Error);
            return Ok(());
        }

        let result = if state.allow_branches {
            // simple -A only (creates branch point)
            match state.rebase_type {
                RebaseType::Single => commands::rebase::single(source, &dest),
                RebaseType::WithDescendants => commands::rebase::with_descendants(source, &dest),
            }
        } else {
            // clean inline: try to insert between dest and its first child
            match (state.rebase_type, self.get_first_child(&dest)) {
                (RebaseType::Single, Ok(Some(next))) => {
                    commands::rebase::single_with_next(source, &dest, &next)
                }
                (RebaseType::Single, _) => commands::rebase::single(source, &dest),
                (RebaseType::WithDescendants, Ok(Some(next))) => {
                    commands::rebase::with_descendants_and_next(source, &dest, &next)
                }
                (RebaseType::WithDescendants, _) => {
                    commands::rebase::with_descendants(source, &dest)
                }
            }
        };

        match result {
            Ok(_) => {
                // store operation for undo
                self.last_op = Some(state.op_before.clone());

                // check for conflicts
                let has_conflicts = self.check_conflicts();

                self.mode = ModeState::Normal;
                let _ = self.refresh_tree();

                if has_conflicts {
                    self.set_status(
                        "Rebase created conflicts. Press u to undo",
                        MessageKind::Warning,
                    );
                } else {
                    self.set_status("Rebase complete", MessageKind::Success);
                }
            }
            Err(e) => {
                self.set_status(&format!("Rebase failed: {e}"), MessageKind::Error);
            }
        }
        Ok(())
    }

    fn check_conflicts(&self) -> bool {
        commands::has_conflicts().unwrap_or(false)
    }

    fn cancel_rebase(&mut self) {
        self.mode = ModeState::Normal;
    }

    fn quick_rebase_onto_trunk(&mut self, rebase_type: RebaseType) -> Result<()> {
        let source = self.current_rev();
        if source.is_empty() {
            self.set_status("No revision selected", MessageKind::Error);
            return Ok(());
        }

        let short_rev = &source[..8.min(source.len())];
        let (mode_flag, message) = match rebase_type {
            RebaseType::Single => ("-r", format!("Rebase {} onto trunk?", short_rev)),
            RebaseType::WithDescendants => (
                "-s",
                format!("Rebase {} and descendants onto trunk?", short_rev),
            ),
        };

        let cmd_preview = format!(
            "jj rebase {} {} -d trunk() --skip-emptied",
            mode_flag, short_rev
        );

        self.mode = ModeState::Confirming(ConfirmState {
            action: ConfirmAction::RebaseOntoTrunk(rebase_type),
            message,
            revs: vec![cmd_preview],
        });
        Ok(())
    }

    fn undo_last_operation(&mut self) -> Result<()> {
        if let Some(ref op_id) = self.last_op.take() {
            match commands::restore_op(op_id) {
                Ok(_) => {
                    self.set_status("Operation undone", MessageKind::Success);
                    let _ = self.refresh_tree();
                }
                Err(e) => {
                    self.set_status(&format!("Undo failed: {e}"), MessageKind::Error);
                }
            }
        } else {
            self.set_status("Nothing to undo", MessageKind::Warning);
        }
        Ok(())
    }

    // Bookmark operations

    fn enter_move_bookmark_mode(&mut self) -> Result<()> {
        let Some(node) = self.tree.current_node() else {
            self.set_status("No revision selected", MessageKind::Error);
            return Ok(());
        };

        // If no bookmarks on this revision, show picker to select any bookmark to move here
        if node.bookmarks.is_empty() {
            return self.enter_bookmark_picker_mode(node.change_id.clone());
        }

        // if multiple bookmarks, show selection dialog
        if node.bookmarks.len() > 1 {
            self.mode = ModeState::BookmarkSelect(BookmarkSelectState {
                bookmarks: node.bookmark_names(),
                selected_index: 0,
                target_rev: node.change_id.clone(),
                action: BookmarkSelectAction::Move,
            });
            return Ok(());
        }

        let bookmark_name = node.bookmarks[0].name.clone();
        let op_before = self.get_current_operation_id().unwrap_or_default();

        self.mode = ModeState::MovingBookmark(MovingBookmarkState {
            bookmark_name,
            dest_cursor: self.tree.cursor,
            op_before,
        });
        Ok(())
    }

    fn handle_moving_bookmark_key(&mut self, code: KeyCode) {
        let ModeState::MovingBookmark(ref state) = self.mode else {
            self.mode = ModeState::Normal;
            return;
        };
        let state = state.clone();

        match code {
            KeyCode::Char('j') | KeyCode::Down => self.move_bookmark_dest_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_bookmark_dest_up(),
            KeyCode::Enter => {
                let _ = self.execute_bookmark_move(&state);
            }
            KeyCode::Esc => self.cancel_bookmark_move(),
            _ => {}
        }
    }

    fn move_bookmark_dest_up(&mut self) {
        if let ModeState::MovingBookmark(ref mut state) = self.mode {
            if state.dest_cursor > 0 {
                state.dest_cursor -= 1;
            }
        }
    }

    fn move_bookmark_dest_down(&mut self) {
        if let ModeState::MovingBookmark(ref mut state) = self.mode {
            let max = self.tree.visible_count().saturating_sub(1);
            if state.dest_cursor < max {
                state.dest_cursor += 1;
            }
        }
    }

    fn execute_bookmark_move(&mut self, state: &MovingBookmarkState) -> Result<()> {
        let Some(dest) = self.get_rev_at_cursor(state.dest_cursor) else {
            self.set_status("Invalid destination", MessageKind::Error);
            return Ok(());
        };

        let name = &state.bookmark_name;

        // Check if this move would be backwards
        if self.is_bookmark_move_backwards(name, &dest) {
            // Show confirmation dialog for backwards move
            self.mode = ModeState::Confirming(ConfirmState {
                action: ConfirmAction::MoveBookmarkBackwards {
                    bookmark_name: name.clone(),
                    dest_rev: dest.clone(),
                    op_before: state.op_before.clone(),
                },
                message: format!(
                    "Move bookmark '{name}' backwards to {}? (This moves the bookmark to an ancestor)",
                    &dest[..8.min(dest.len())]
                ),
                revs: vec![],
            });
            return Ok(());
        }

        // Normal forward move
        self.do_bookmark_move(name, &dest, &state.op_before, false);

        self.mode = ModeState::Normal;
        Ok(())
    }

    /// Execute the actual bookmark move, optionally with --allow-backwards
    fn do_bookmark_move(
        &mut self,
        name: &str,
        dest: &str,
        op_before: &str,
        allow_backwards: bool,
    ) {
        let result = if allow_backwards {
            commands::bookmark::set_allow_backwards(name, dest)
        } else {
            commands::bookmark::set(name, dest)
        };

        match result {
            Ok(_) => {
                self.last_op = Some(op_before.to_string());
                let _ = self.refresh_tree();
                self.set_status(
                    &format!("Moved bookmark '{name}' to {}", &dest[..8.min(dest.len())]),
                    MessageKind::Success,
                );
            }
            Err(e) => {
                let error_details = format!("{e}");
                self.set_error_with_details("Move bookmark failed", &error_details);
            }
        }
    }

    fn cancel_bookmark_move(&mut self) {
        self.mode = ModeState::Normal;
    }

    fn enter_create_bookmark(&mut self) -> Result<()> {
        let rev = self.current_rev();
        if rev.is_empty() {
            self.set_status("No revision selected", MessageKind::Error);
            return Ok(());
        }

        self.mode = ModeState::BookmarkInput(BookmarkInputState {
            name: String::new(),
            cursor: 0,
            target_rev: rev.clone(),
            deleting: false,
        });
        self.set_status(&format!("Creating bookmark at {}", &rev[..8.min(rev.len())]), MessageKind::Success);
        Ok(())
    }

    fn handle_bookmark_input_key(&mut self, key: event::KeyEvent) {
        if let ModeState::BookmarkInput(ref mut state) = self.mode {
            match key.code {
                KeyCode::Enter => {
                    let name = state.name.clone();
                    let target = state.target_rev.clone();
                    let deleting = state.deleting;
                    self.execute_bookmark_input(&name, &target, deleting);
                }
                KeyCode::Esc => {
                    self.mode = ModeState::Normal;
                }
                KeyCode::Char(c) => {
                    state.name.insert(state.cursor, c);
                    state.cursor += c.len_utf8();
                }
                KeyCode::Backspace => {
                    if state.cursor > 0 {
                        let prev = state.name[..state.cursor]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        state.name.remove(prev);
                        state.cursor = prev;
                    }
                }
                KeyCode::Delete => {
                    if state.cursor < state.name.len() {
                        state.name.remove(state.cursor);
                    }
                }
                KeyCode::Left => {
                    if state.cursor > 0 {
                        state.cursor = state.name[..state.cursor]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                    }
                }
                KeyCode::Right => {
                    if state.cursor < state.name.len() {
                        state.cursor = state.name[state.cursor..]
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| state.cursor + i)
                            .unwrap_or(state.name.len());
                    }
                }
                _ => {}
            }
        }
    }

    fn execute_bookmark_input(&mut self, name: &str, target: &str, deleting: bool) {
        if name.is_empty() {
            self.set_status("Bookmark name cannot be empty", MessageKind::Error);
            self.mode = ModeState::Normal;
            return;
        }

        let op_before = self.get_current_operation_id().unwrap_or_default();

        let result = if deleting {
            commands::bookmark::delete(name)
        } else {
            commands::bookmark::set(name, target)
        };

        match result {
            Ok(()) => {
                self.last_op = Some(op_before);
                let _ = self.refresh_tree();
                let action = if deleting { "Deleted" } else { "Set" };
                self.set_status(&format!("{action} bookmark '{name}'"), MessageKind::Success);
            }
            Err(e) => {
                let action = if deleting { "Delete" } else { "Set" };
                let error_details = format!("{e}");
                self.set_error_with_details(&format!("{action} bookmark failed"), &error_details);
            }
        }

        self.mode = ModeState::Normal;
    }

    fn handle_bookmark_select_key(&mut self, code: KeyCode) {
        let ModeState::BookmarkSelect(ref state) = self.mode else {
            self.mode = ModeState::Normal;
            return;
        };
        let state = state.clone();

        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                if let ModeState::BookmarkSelect(ref mut s) = self.mode {
                    if s.selected_index < s.bookmarks.len().saturating_sub(1) {
                        s.selected_index += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let ModeState::BookmarkSelect(ref mut s) = self.mode {
                    if s.selected_index > 0 {
                        s.selected_index -= 1;
                    }
                }
            }
            KeyCode::Enter => {
                let bookmark = state.bookmarks[state.selected_index].clone();

                match state.action {
                    BookmarkSelectAction::Move => {
                        let op_before = self.get_current_operation_id().unwrap_or_default();
                        self.mode = ModeState::MovingBookmark(MovingBookmarkState {
                            bookmark_name: bookmark,
                            dest_cursor: self.tree.cursor,
                            op_before,
                        });
                    }
                    BookmarkSelectAction::Delete => {
                        let op_before = self.get_current_operation_id().unwrap_or_default();
                        match commands::bookmark::delete(&bookmark) {
                            Ok(_) => {
                                self.last_op = Some(op_before);
                                let _ = self.refresh_tree();
                                self.set_status(
                                    &format!("Deleted bookmark '{bookmark}'"),
                                    MessageKind::Success,
                                );
                            }
                            Err(e) => {
                                let error_details = format!("{e}");
                                self.set_error_with_details(
                                    "Delete bookmark failed",
                                    &error_details,
                                );
                            }
                        }
                        self.mode = ModeState::Normal;
                    }
                }
            }
            KeyCode::Esc => {
                self.mode = ModeState::Normal;
            }
            _ => {}
        }
    }

    pub fn current_has_bookmark(&self) -> bool {
        self.tree
            .current_node()
            .map(|n| !n.bookmarks.is_empty())
            .unwrap_or(false)
    }

    // Bookmark picker - select any bookmark to move to current revision

    fn enter_bookmark_picker_mode(&mut self, target_rev: String) -> Result<()> {
        let jj_repo = JjRepo::load(None)?;
        let all_bookmarks = jj_repo.all_local_bookmarks();

        if all_bookmarks.is_empty() {
            self.set_status("No bookmarks in repository", MessageKind::Warning);
            return Ok(());
        }

        self.mode = ModeState::BookmarkPicker(BookmarkPickerState {
            all_bookmarks,
            filter: String::new(),
            filter_cursor: 0,
            selected_index: 0,
            target_rev,
            action: BookmarkSelectAction::Move,
        });
        Ok(())
    }

    fn delete_bookmark_with_picker(&mut self) -> Result<()> {
        let jj_repo = JjRepo::load(None)?;
        let mut all_bookmarks = jj_repo.all_local_bookmarks();

        if all_bookmarks.is_empty() {
            self.set_status("No bookmarks in repository", MessageKind::Warning);
            return Ok(());
        }

        // get bookmarks on current commit to prioritize them
        let current_bookmarks: Vec<String> = self
            .tree
            .current_node()
            .map(|n| n.bookmark_names())
            .unwrap_or_default();

        // reorder: current commit's bookmarks first, then the rest
        if !current_bookmarks.is_empty() {
            all_bookmarks.retain(|b| !current_bookmarks.contains(b));
            let mut reordered = current_bookmarks;
            reordered.extend(all_bookmarks);
            all_bookmarks = reordered;
        }

        let target_rev = self
            .tree
            .current_node()
            .map(|n| n.change_id.clone())
            .unwrap_or_default();

        self.mode = ModeState::BookmarkPicker(BookmarkPickerState {
            all_bookmarks,
            filter: String::new(),
            filter_cursor: 0,
            selected_index: 0,
            target_rev,
            action: BookmarkSelectAction::Delete,
        });
        Ok(())
    }

    fn handle_bookmark_picker_key(&mut self, key: event::KeyEvent) {
        let ModeState::BookmarkPicker(ref state) = self.mode else {
            self.mode = ModeState::Normal;
            return;
        };
        let state = state.clone();

        match key.code {
            KeyCode::Esc => {
                self.mode = ModeState::Normal;
            }
            KeyCode::Enter => {
                let filtered = state.filtered_bookmarks();
                if let Some(bookmark) = filtered.get(state.selected_index) {
                    let bookmark_name = (*bookmark).clone();
                    let target_rev = state.target_rev.clone();
                    let action = state.action;

                    match action {
                        BookmarkSelectAction::Move => {
                            if self.is_bookmark_move_backwards(&bookmark_name, &target_rev) {
                                let op_before = self.get_current_operation_id().unwrap_or_default();
                                self.mode = ModeState::Confirming(ConfirmState {
                                    action: ConfirmAction::MoveBookmarkBackwards {
                                        bookmark_name: bookmark_name.clone(),
                                        dest_rev: target_rev.clone(),
                                        op_before,
                                    },
                                    message: format!(
                                        "Move bookmark '{}' backwards to {}? (This moves the bookmark to an ancestor)",
                                        bookmark_name,
                                        &target_rev[..8.min(target_rev.len())]
                                    ),
                                    revs: vec![],
                                });
                            } else {
                                let op_before = self.get_current_operation_id().unwrap_or_default();
                                self.do_bookmark_move(&bookmark_name, &target_rev, &op_before, false);
                                self.mode = ModeState::Normal;
                            }
                        }
                        BookmarkSelectAction::Delete => {
                            let op_before = self.get_current_operation_id().unwrap_or_default();
                            match commands::bookmark::delete(&bookmark_name) {
                                Ok(_) => {
                                    self.last_op = Some(op_before);
                                    let _ = self.refresh_tree();
                                    self.set_status(
                                        &format!("Deleted bookmark '{bookmark_name}'"),
                                        MessageKind::Success,
                                    );
                                }
                                Err(e) => {
                                    let error_details = format!("{e}");
                                    self.set_error_with_details("Delete bookmark failed", &error_details);
                                }
                            }
                            self.mode = ModeState::Normal;
                        }
                    }
                } else {
                    self.set_status("No bookmark selected", MessageKind::Warning);
                    self.mode = ModeState::Normal;
                }
            }
            KeyCode::Down => {
                if let ModeState::BookmarkPicker(ref mut s) = self.mode {
                    let filtered_count = s.filtered_bookmarks().len();
                    if s.selected_index < filtered_count.saturating_sub(1) {
                        s.selected_index += 1;
                    }
                }
            }
            KeyCode::Up => {
                if let ModeState::BookmarkPicker(ref mut s) = self.mode {
                    if s.selected_index > 0 {
                        s.selected_index -= 1;
                    }
                }
            }
            KeyCode::Char(c) => {
                if let ModeState::BookmarkPicker(ref mut s) = self.mode {
                    s.filter.insert(s.filter_cursor, c);
                    s.filter_cursor += c.len_utf8();
                    s.selected_index = 0; // reset selection when filter changes
                }
            }
            KeyCode::Backspace => {
                if let ModeState::BookmarkPicker(ref mut s) = self.mode {
                    if s.filter_cursor > 0 {
                        let prev = s.filter[..s.filter_cursor]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        s.filter.remove(prev);
                        s.filter_cursor = prev;
                        s.selected_index = 0; // reset selection when filter changes
                    }
                }
            }
            _ => {}
        }
    }

    // Squash operations

    fn enter_squash_mode(&mut self) -> Result<()> {
        let source_rev = self.current_rev();
        if source_rev.is_empty() {
            self.set_status("No revision selected", MessageKind::Error);
            return Ok(());
        }

        let op_before = self.get_current_operation_id().unwrap_or_default();

        // start with cursor at parent (same logic as rebase mode)
        let current = self.tree.cursor;
        let source_struct_depth = self
            .tree
            .visible_entries
            .get(current)
            .map(|e| self.tree.nodes[e.node_index].depth)
            .unwrap_or(0);

        // find source's parent: closest entry above with smaller structural depth
        let mut initial_cursor = current.saturating_sub(1);
        while initial_cursor > 0 {
            let entry = &self.tree.visible_entries[initial_cursor];
            let node = &self.tree.nodes[entry.node_index];
            if node.depth < source_struct_depth {
                break;
            }
            initial_cursor -= 1;
        }

        self.mode = ModeState::Squashing(SquashState {
            source_rev,
            dest_cursor: initial_cursor,
            op_before,
        });
        Ok(())
    }

    fn handle_squashing_key(&mut self, code: KeyCode) {
        let ModeState::Squashing(ref state) = self.mode else {
            self.mode = ModeState::Normal;
            return;
        };
        let state = state.clone();

        match code {
            KeyCode::Char('j') | KeyCode::Down => self.move_squash_dest_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_squash_dest_up(),
            KeyCode::Enter => {
                let _ = self.execute_squash(&state);
            }
            KeyCode::Esc => self.cancel_squash(),
            _ => {}
        }
    }

    fn move_squash_dest_up(&mut self) {
        if let ModeState::Squashing(ref mut state) = self.mode {
            if state.dest_cursor > 0 {
                state.dest_cursor -= 1;
            }
        }
    }

    fn move_squash_dest_down(&mut self) {
        if let ModeState::Squashing(ref mut state) = self.mode {
            let max = self.tree.visible_count().saturating_sub(1);
            if state.dest_cursor < max {
                state.dest_cursor += 1;
            }
        }
    }

    fn execute_squash(&mut self, state: &SquashState) -> Result<()> {
        let source = &state.source_rev;
        let Some(target) = self.get_rev_at_cursor(state.dest_cursor) else {
            self.set_status("Invalid target", MessageKind::Error);
            return Ok(());
        };

        if *source == target {
            self.set_status("Cannot squash into self", MessageKind::Error);
            return Ok(());
        }

        // set pending squash state - the actual command runs in run_loop()
        // because jj squash may open an editor when both revisions have descriptions
        self.pending_squash = Some(PendingSquash {
            source_rev: source.clone(),
            target_rev: target,
            op_before: state.op_before.clone(),
        });
        self.mode = ModeState::Normal;
        Ok(())
    }

    fn cancel_squash(&mut self) {
        self.mode = ModeState::Normal;
    }

    /// Compute indices of entries that will move during rebase
    /// For 's' mode: source + all descendants
    /// For 'r' mode: only source
    pub fn compute_moving_indices(&self) -> HashSet<usize> {
        let ModeState::Rebasing(ref state) = self.mode else {
            return HashSet::new();
        };

        let mut indices = HashSet::new();
        let mut in_source_tree = false;
        let mut source_struct_depth = 0usize;

        for (idx, entry) in self.tree.visible_entries.iter().enumerate() {
            let node = &self.tree.nodes[entry.node_index];

            if node.change_id == state.source_rev {
                indices.insert(idx);
                if state.rebase_type == RebaseType::WithDescendants {
                    in_source_tree = true;
                    source_struct_depth = node.depth;
                }
            } else if in_source_tree {
                if node.depth > source_struct_depth {
                    indices.insert(idx);
                } else {
                    break;
                }
            }
        }

        indices
    }
}

fn syntect_to_ratatui_color(style: SyntectStyle) -> Color {
    Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

fn parse_diff(output: &str, ss: &SyntaxSet, ts: &ThemeSet) -> Vec<DiffLine> {
    let theme = &ts.themes["base16-eighties.dark"];
    let plain_text = ss.find_syntax_plain_text();

    let mut current_file: Option<String> = None;
    let mut lines = Vec::new();

    for line in output.lines() {
        let (kind, code_content) = if line.starts_with("diff --git") {
            // extract filename from "diff --git a/path/file.rs b/path/file.rs"
            if let Some(b_path) = line.split(" b/").nth(1) {
                current_file = Some(b_path.to_string());
            }
            (DiffLineKind::FileHeader, None)
        } else if line.starts_with("+++") || line.starts_with("---") {
            (DiffLineKind::FileHeader, None)
        } else if line.starts_with("@@") {
            (DiffLineKind::Hunk, None)
        } else if let Some(rest) = line.strip_prefix('+') {
            (DiffLineKind::Added, Some(rest))
        } else if let Some(rest) = line.strip_prefix('-') {
            (DiffLineKind::Removed, Some(rest))
        } else if let Some(rest) = line.strip_prefix(' ') {
            (DiffLineKind::Context, Some(rest))
        } else {
            (DiffLineKind::Context, Some(line))
        };

        let spans = if let Some(code) = code_content {
            let prefix = match kind {
                DiffLineKind::Added => "+",
                DiffLineKind::Removed => "-",
                DiffLineKind::Context => " ",
                _ => "",
            };

            let prefix_color = match kind {
                DiffLineKind::Added => Color::Green,
                DiffLineKind::Removed => Color::Red,
                _ => Color::DarkGray,
            };

            // try syntect highlighting
            let syntax = current_file.as_ref().and_then(|f| {
                std::path::Path::new(f)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| ss.find_syntax_by_extension(ext))
            });

            let code_spans = if let Some(syn) = syntax {
                let mut highlighter = syntect::easy::HighlightLines::new(syn, theme);
                highlighter.highlight_line(code, ss).ok().map(|ranges| {
                    ranges
                        .into_iter()
                        .map(|(style, text)| StyledSpan {
                            text: text.to_string(),
                            fg: syntect_to_ratatui_color(style),
                        })
                        .collect::<Vec<_>>()
                })
            } else {
                None
            };

            // fall back to plain text
            let code_spans = code_spans.unwrap_or_else(|| {
                let mut highlighter = syntect::easy::HighlightLines::new(plain_text, theme);
                highlighter
                    .highlight_line(code, ss)
                    .map(|ranges| {
                        ranges
                            .into_iter()
                            .map(|(style, text)| StyledSpan {
                                text: text.to_string(),
                                fg: syntect_to_ratatui_color(style),
                            })
                            .collect()
                    })
                    .unwrap_or_else(|_| {
                        vec![StyledSpan {
                            text: code.to_string(),
                            fg: Color::White,
                        }]
                    })
            });

            let mut result = vec![StyledSpan {
                text: prefix.to_string(),
                fg: prefix_color,
            }];
            result.extend(code_spans);
            result
        } else {
            // non-code lines (headers, hunks)
            let color = match kind {
                DiffLineKind::FileHeader => Color::Yellow,
                DiffLineKind::Hunk => Color::Cyan,
                _ => Color::White,
            };
            vec![StyledSpan {
                text: line.to_string(),
                fg: color,
            }]
        };

        lines.push(DiffLine { spans, kind });
    }

    lines
}
