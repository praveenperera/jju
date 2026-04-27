use super::{App, AppOptions};
use crate::cmd::jj_tui::keybindings;
use crate::cmd::jj_tui::state::{MessageKind, ModeState, StatusMessage};
use crate::cmd::jj_tui::tree::{TreeLoadScope, TreeState};
use crate::jj_lib_helpers::JjRepo;
use eyre::Result;
use log::info;
use std::time::Instant;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub(super) fn new_app(options: AppOptions) -> Result<App> {
    let startup_started_at = Instant::now();
    let keybindings_warning = keybindings::initialize();
    let repo_path = std::env::current_dir()?;
    let jj_repo = JjRepo::load(Some(&repo_path))?;
    let load_scope = startup_load_scope(options);
    let mut tree = TreeState::load_with_scope(&jj_repo, "trunk()", load_scope)?;
    apply_startup_options(&mut tree, options);
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let app = App {
        tree,
        mode: ModeState::Normal,
        should_quit: false,
        split_view: false,
        diff_stats_cache: std::collections::HashMap::new(),
        status_message: keybindings_warning.map(|warning| {
            StatusMessage::with_duration(
                warning,
                MessageKind::Warning,
                keybindings::warning_duration(),
            )
        }),
        last_op: None,
        pending_key: None,
        syntax_set,
        theme_set,
        repo_path,
        row_data_loader: Default::default(),
    };
    info!("Initialized jj_tui in {:?}", startup_started_at.elapsed());

    Ok(app)
}

fn startup_load_scope(options: AppOptions) -> TreeLoadScope {
    if options.start_in_neighborhood {
        TreeLoadScope::Neighborhood
    } else {
        TreeLoadScope::Stack
    }
}

pub(super) fn apply_startup_options(tree: &mut TreeState, options: AppOptions) {
    if options.start_in_neighborhood {
        tree.jump_to_working_copy();
        tree.enable_neighborhood();
    }
}
