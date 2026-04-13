use super::{KeyFormat, PrefixMenuView, keys_for_label};
use crate::cmd::jj_tui::keybindings::ModeId;

pub fn prefix_menu(mode: ModeId, pending: char) -> Option<PrefixMenuView> {
    let title = super::super::prefix_title(pending)?;
    let mut items = Vec::new();
    for command in super::super::commands() {
        if command.mode != mode || !command.uses_prefix(pending) {
            continue;
        }
        let Some(key) = keys_for_label(
            mode,
            Some(pending),
            command.label,
            false,
            KeyFormat::SecondKeyOnly,
        )
        .into_iter()
        .next() else {
            continue;
        };
        items.push((key, command.label));
    }
    Some(PrefixMenuView { title, items })
}
