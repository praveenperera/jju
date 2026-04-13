use crate::cmd::jj_tui::keybindings::{KeyPattern, ModeId};

pub(super) fn describe_binding_key(pending: Option<char>, key: KeyPattern) -> String {
    match (pending, key) {
        (
            Some(prefix),
            KeyPattern::Exact {
                code,
                required_mods,
            },
        ) => {
            format!("{prefix} {}", describe_key(code, required_mods))
        }
        (
            None,
            KeyPattern::Exact {
                code,
                required_mods,
            },
        ) => describe_key(code, required_mods),
        (_, KeyPattern::AnyChar) => "AnyChar".to_string(),
    }
}

pub(super) fn mode_name(mode: ModeId) -> &'static str {
    match mode {
        ModeId::Normal => "normal",
        ModeId::Help => "help",
        ModeId::Diff => "diff",
        ModeId::Confirm => "confirm",
        ModeId::Selecting => "selecting",
        ModeId::Rebase => "rebase",
        ModeId::Squash => "squash",
        ModeId::MovingBookmark => "moving_bookmark",
        ModeId::BookmarkSelect => "bookmark_select",
        ModeId::BookmarkPicker => "bookmark_picker",
        ModeId::PushSelect => "push_select",
        ModeId::Conflicts => "conflicts",
    }
}

fn describe_key(
    code: ratatui::crossterm::event::KeyCode,
    mods: ratatui::crossterm::event::KeyModifiers,
) -> String {
    if mods.contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
        && let ratatui::crossterm::event::KeyCode::Char(ch) = code
    {
        return format!("Ctrl+{ch}");
    }

    match code {
        ratatui::crossterm::event::KeyCode::Char(' ') => "Space".to_string(),
        ratatui::crossterm::event::KeyCode::Char(ch) => ch.to_string(),
        other => format!("{other:?}"),
    }
}
