use super::display::{KeyFormat, first_key, join_keys, keys_for_label};
use super::{DisplayKind, ModeId};

#[derive(Debug, Clone, Copy)]
pub struct StatusHintContext {
    pub mode: ModeId,
    pub has_selection: bool,
    pub has_focus: bool,
    pub current_has_bookmark: bool,
    pub rebase_allow_branches: Option<bool>,
}

pub fn status_bar_hints(ctx: &StatusHintContext) -> String {
    match ctx.mode {
        ModeId::Normal => {
            if ctx.has_selection {
                join_segments(&[
                    kv(ModeId::Normal, None, "abandon", "abandon"),
                    kv(ModeId::Normal, None, "toggle", "toggle"),
                    kv(ModeId::Normal, None, "esc", "clear"),
                ])
            } else if ctx.has_focus {
                join_segments(&[
                    kv(ModeId::Normal, None, "zoom", "unfocus"),
                    kv(ModeId::Normal, None, "full", "toggle-full"),
                    kv(ModeId::Normal, None, "help", "help"),
                    kv(ModeId::Normal, None, "quit", "quit"),
                ])
            } else if ctx.current_has_bookmark {
                join_segments(&[
                    kv(ModeId::Normal, None, "push", "push"),
                    kv(ModeId::Normal, None, "stack-sync", "sync"),
                    kv(ModeId::Normal, None, "bookmark", "bookmark"),
                    kv(ModeId::Normal, None, "rebase-single", "rebase"),
                    kv(ModeId::Normal, None, "help", "help"),
                    kv(ModeId::Normal, None, "quit", "quit"),
                ])
            } else {
                join_segments(&[
                    format!(
                        "{}/{}:rebase",
                        key_for_hint(ModeId::Normal, None, "rebase-single"),
                        key_for_hint(ModeId::Normal, None, "rebase-desc")
                    ),
                    kv(ModeId::Normal, None, "trunk-single", "trunk"),
                    kv(ModeId::Normal, None, "desc", "desc"),
                    kv(ModeId::Normal, None, "stack-sync", "sync"),
                    kv(ModeId::Normal, None, "bookmark", "bookmark"),
                    kv(ModeId::Normal, None, "git", "git"),
                    kv(ModeId::Normal, None, "nav", "nav"),
                    kv(ModeId::Normal, None, "help", "help"),
                    kv(ModeId::Normal, None, "quit", "quit"),
                ])
            }
        }
        ModeId::Help => join_segments(&[
            format!(
                "{}/{}:scroll",
                key_for_hint(ModeId::Help, None, "scroll-down"),
                key_for_hint(ModeId::Help, None, "scroll-up")
            ),
            {
                let keys = keys_for_label(ModeId::Help, None, "close", true, KeyFormat::Space);
                format!("{}:close", join_keys(&keys, "/"))
            },
        ]),
        ModeId::Diff => join_segments(&[
            format!(
                "{}/{}:scroll",
                key_for_hint(ModeId::Diff, None, "scroll-down"),
                key_for_hint(ModeId::Diff, None, "scroll-up")
            ),
            format!(
                "{}/{}:page",
                key_for_hint(ModeId::Diff, None, "page-down"),
                key_for_hint(ModeId::Diff, None, "page-up")
            ),
            format!(
                "{}/{}:top/bottom",
                key_for_hint_chord(ModeId::Diff, 'z', "top"),
                key_for_hint_chord(ModeId::Diff, 'z', "bottom")
            ),
            {
                let keys = keys_for_label(ModeId::Diff, None, "close", true, KeyFormat::Space);
                format!("{}:close", join_keys(&keys, "/"))
            },
        ]),
        ModeId::Confirm => join_segments(&[
            {
                let keys = keys_for_label(ModeId::Confirm, None, "yes", true, KeyFormat::Space);
                format!("{}:yes", join_keys(&keys, "/"))
            },
            {
                let keys = keys_for_label(ModeId::Confirm, None, "no", true, KeyFormat::Space);
                format!("{}:no", join_keys(&keys, "/"))
            },
        ]),
        ModeId::Selecting => join_segments(&[
            format!(
                "{}/{}:extend",
                key_for_hint(ModeId::Selecting, None, "down"),
                key_for_hint(ModeId::Selecting, None, "up")
            ),
            kv(ModeId::Selecting, None, "abandon", "abandon"),
            kv(ModeId::Selecting, None, "exit", "exit"),
        ]),
        ModeId::Rebase => {
            let branches_label = if ctx.rebase_allow_branches.unwrap_or(false) {
                "inline"
            } else {
                "branch"
            };
            join_segments(&[
                format!(
                    "{}/{}:dest",
                    key_for_hint(ModeId::Rebase, None, "dest-down"),
                    key_for_hint(ModeId::Rebase, None, "dest-up")
                ),
                kv(ModeId::Rebase, None, "branches", branches_label),
                kv(ModeId::Rebase, None, "run", "run"),
                kv(ModeId::Rebase, None, "cancel", "cancel"),
            ])
        }
        ModeId::Squash => join_segments(&[
            format!(
                "{}/{}:dest",
                key_for_hint(ModeId::Squash, None, "dest-down"),
                key_for_hint(ModeId::Squash, None, "dest-up")
            ),
            kv(ModeId::Squash, None, "run", "run"),
            kv(ModeId::Squash, None, "cancel", "cancel"),
        ]),
        ModeId::MovingBookmark => join_segments(&[
            format!(
                "{}/{}:dest",
                key_for_hint(ModeId::MovingBookmark, None, "dest-down"),
                key_for_hint(ModeId::MovingBookmark, None, "dest-up")
            ),
            kv(ModeId::MovingBookmark, None, "run", "run"),
            kv(ModeId::MovingBookmark, None, "cancel", "cancel"),
        ]),
        ModeId::BookmarkSelect => join_segments(&[
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::BookmarkSelect, None, "down"),
                key_for_hint(ModeId::BookmarkSelect, None, "up")
            ),
            kv(ModeId::BookmarkSelect, None, "select", "select"),
            kv(ModeId::BookmarkSelect, None, "cancel", "cancel"),
        ]),
        ModeId::BookmarkPicker => join_segments(&[
            "type:filter".to_string(),
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::BookmarkPicker, None, "up"),
                key_for_hint(ModeId::BookmarkPicker, None, "down")
            ),
            kv(ModeId::BookmarkPicker, None, "confirm", "select"),
            kv(ModeId::BookmarkPicker, None, "cancel", "cancel"),
        ]),
        ModeId::PushSelect => join_segments(&[
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::PushSelect, None, "up"),
                key_for_hint(ModeId::PushSelect, None, "down")
            ),
            kv(ModeId::PushSelect, None, "toggle", "toggle"),
            kv(ModeId::PushSelect, None, "all", "all"),
            kv(ModeId::PushSelect, None, "none", "none"),
            kv(ModeId::PushSelect, None, "push", "push"),
            kv(ModeId::PushSelect, None, "cancel", "cancel"),
        ]),
        ModeId::Conflicts => join_segments(&[
            format!(
                "{}/{}:nav",
                key_for_hint(ModeId::Conflicts, None, "down"),
                key_for_hint(ModeId::Conflicts, None, "up")
            ),
            kv(ModeId::Conflicts, None, "resolve", "resolve"),
            {
                let keys = keys_for_label(ModeId::Conflicts, None, "exit", true, KeyFormat::Space);
                format!("{}:exit", join_keys(&keys, "/"))
            },
        ]),
    }
}

fn key_for_hint(mode: ModeId, pending: Option<char>, label: &str) -> String {
    first_key(mode, pending, label, DisplayKind::Primary)
        .unwrap_or_else(|| "?".to_string())
        .split_whitespace()
        .last()
        .unwrap_or("?")
        .to_string()
}

fn key_for_hint_chord(mode: ModeId, prefix: char, label: &str) -> String {
    keys_for_label(mode, Some(prefix), label, false, KeyFormat::Concat)
        .into_iter()
        .next()
        .unwrap_or_else(|| "?".to_string())
}

fn kv(mode: ModeId, pending: Option<char>, label: &str, value: &str) -> String {
    format!("{}:{value}", key_for_hint(mode, pending, label))
}

fn join_segments(segments: &[String]) -> String {
    segments.join("  ")
}
