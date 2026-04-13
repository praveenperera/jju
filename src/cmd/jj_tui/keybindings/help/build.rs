use super::HelpItemView;
use crate::cmd::jj_tui::keybindings::display::{KeyFormat, display_keys_for_command, join_keys};

pub(super) fn build_help_sections() -> std::collections::HashMap<&'static str, Vec<HelpItemView>> {
    let mut sections = std::collections::HashMap::new();

    for command in crate::cmd::jj_tui::keybindings::commands() {
        let Some(help) = &command.help else {
            continue;
        };

        let keys = display_keys_for_command(
            command.mode,
            command.label,
            help.include_aliases,
            KeyFormat::Space,
        );
        if keys.is_empty() {
            continue;
        }
        let keys = if help.include_aliases {
            join_keys(&keys, "/")
        } else {
            keys[0].clone()
        };

        sections
            .entry(help.section)
            .or_insert_with(Vec::new)
            .push(HelpItemView {
                keys,
                description: help.description,
            });
    }

    sections
}
