use super::catalog::command_id as cmd;
use super::display::{KeyFormat, first_key, first_key_any_pending, join_keys, keys_for_label};
use super::{DisplayKind, ModeId};

#[derive(Debug, Clone, Copy)]
pub struct StatusHintContext {
    pub mode: ModeId,
    pub has_selection: bool,
    pub has_focus: bool,
    pub neighborhood_active: bool,
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
            } else if ctx.neighborhood_active {
                join_segments(&[
                    format!(
                        "{}:full",
                        key_for_hint_any_pending(
                            ModeId::Normal,
                            cmd::NEIGHBORHOOD,
                            KeyFormat::Concat
                        )
                    ),
                    format!(
                        "{}/{}:size",
                        key_for_hint_any_pending(
                            ModeId::Normal,
                            cmd::NEIGHBORHOOD_MORE,
                            KeyFormat::Concat
                        ),
                        key_for_hint_any_pending(
                            ModeId::Normal,
                            cmd::NEIGHBORHOOD_LESS,
                            KeyFormat::Concat
                        )
                    ),
                    kv(ModeId::Normal, None, cmd::DIFF, "diff"),
                    kv(ModeId::Normal, None, cmd::DESC, "desc"),
                    kv(ModeId::Normal, None, cmd::HELP, "help"),
                    kv(ModeId::Normal, None, cmd::QUIT, "quit"),
                ])
            } else if ctx.has_focus {
                join_segments(&[
                    kv(ModeId::Normal, None, cmd::ZOOM, "unfocus"),
                    kv(ModeId::Normal, None, cmd::FULL, "toggle-full"),
                    kv(ModeId::Normal, None, cmd::HELP, "help"),
                    kv(ModeId::Normal, None, cmd::QUIT, "quit"),
                ])
            } else if ctx.current_has_bookmark {
                join_segments(&[
                    kv(ModeId::Normal, None, cmd::PUSH, "push"),
                    kv(ModeId::Normal, None, cmd::STACK_SYNC, "sync"),
                    kv(ModeId::Normal, None, cmd::BOOKMARK, "bookmark"),
                    kv(ModeId::Normal, None, cmd::REBASE_SINGLE, "rebase"),
                    kv(ModeId::Normal, None, cmd::HELP, "help"),
                    kv(ModeId::Normal, None, cmd::QUIT, "quit"),
                ])
            } else {
                join_segments(&[
                    format!(
                        "{}/{}:rebase",
                        key_for_hint(ModeId::Normal, None, cmd::REBASE_SINGLE),
                        key_for_hint(ModeId::Normal, None, cmd::REBASE_DESC)
                    ),
                    kv(ModeId::Normal, None, cmd::TRUNK_SINGLE, "trunk"),
                    kv(ModeId::Normal, None, cmd::DESC, "desc"),
                    kv(ModeId::Normal, None, cmd::STACK_SYNC, "sync"),
                    kv(ModeId::Normal, None, cmd::BOOKMARK, "bookmark"),
                    kv(ModeId::Normal, None, cmd::GIT, "git"),
                    kv(ModeId::Normal, None, "nav", "nav"),
                    kv(ModeId::Normal, None, cmd::HELP, "help"),
                    kv(ModeId::Normal, None, cmd::QUIT, "quit"),
                ])
            }
        }
        ModeId::Help => join_segments(&[
            format!(
                "{}/{}:scroll",
                key_for_hint(ModeId::Help, None, cmd::SCROLL_DOWN),
                key_for_hint(ModeId::Help, None, cmd::SCROLL_UP)
            ),
            {
                let keys = keys_for_label(ModeId::Help, None, "close", true, KeyFormat::Space);
                format!("{}:close", join_keys(&keys, "/"))
            },
        ]),
        ModeId::Diff => join_segments(&[
            format!(
                "{}/{}:scroll",
                key_for_hint(ModeId::Diff, None, cmd::SCROLL_DOWN),
                key_for_hint(ModeId::Diff, None, cmd::SCROLL_UP)
            ),
            format!(
                "{}/{}:page",
                key_for_hint(ModeId::Diff, None, cmd::PAGE_DOWN),
                key_for_hint(ModeId::Diff, None, cmd::PAGE_UP)
            ),
            format!(
                "{}/{}:top/bottom",
                key_for_hint_any_pending(ModeId::Diff, cmd::TOP, KeyFormat::Concat),
                key_for_hint_any_pending(ModeId::Diff, cmd::BOTTOM, KeyFormat::Concat)
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
                key_for_hint(ModeId::Selecting, None, cmd::DOWN),
                key_for_hint(ModeId::Selecting, None, cmd::UP)
            ),
            kv(ModeId::Selecting, None, cmd::ABANDON, "abandon"),
            kv(ModeId::Selecting, None, cmd::EXIT, "exit"),
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
                    key_for_hint(ModeId::Rebase, None, cmd::DEST_DOWN),
                    key_for_hint(ModeId::Rebase, None, cmd::DEST_UP)
                ),
                kv(ModeId::Rebase, None, cmd::BRANCHES, branches_label),
                kv(ModeId::Rebase, None, cmd::RUN, "run"),
                kv(ModeId::Rebase, None, cmd::CANCEL, "cancel"),
            ])
        }
        ModeId::Squash => join_segments(&[
            format!(
                "{}/{}:dest",
                key_for_hint(ModeId::Squash, None, cmd::DEST_DOWN),
                key_for_hint(ModeId::Squash, None, cmd::DEST_UP)
            ),
            kv(ModeId::Squash, None, cmd::RUN, "run"),
            kv(ModeId::Squash, None, cmd::CANCEL, "cancel"),
        ]),
        ModeId::MovingBookmark => join_segments(&[
            format!(
                "{}/{}:dest",
                key_for_hint(ModeId::MovingBookmark, None, cmd::DEST_DOWN),
                key_for_hint(ModeId::MovingBookmark, None, cmd::DEST_UP)
            ),
            kv(ModeId::MovingBookmark, None, cmd::RUN, "run"),
            kv(ModeId::MovingBookmark, None, cmd::CANCEL, "cancel"),
        ]),
        ModeId::BookmarkSelect => join_segments(&[
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::BookmarkSelect, None, cmd::DOWN),
                key_for_hint(ModeId::BookmarkSelect, None, cmd::UP)
            ),
            kv(ModeId::BookmarkSelect, None, cmd::SELECT, "select"),
            kv(ModeId::BookmarkSelect, None, cmd::CANCEL, "cancel"),
        ]),
        ModeId::BookmarkPicker => join_segments(&[
            "type:filter".to_string(),
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::BookmarkPicker, None, cmd::UP),
                key_for_hint(ModeId::BookmarkPicker, None, cmd::DOWN)
            ),
            kv(ModeId::BookmarkPicker, None, cmd::CONFIRM, "select"),
            kv(ModeId::BookmarkPicker, None, cmd::CANCEL, "cancel"),
        ]),
        ModeId::PushSelect => join_segments(&[
            format!(
                "{}/{}:navigate",
                key_for_hint(ModeId::PushSelect, None, cmd::UP),
                key_for_hint(ModeId::PushSelect, None, cmd::DOWN)
            ),
            kv(ModeId::PushSelect, None, cmd::TOGGLE, "toggle"),
            kv(ModeId::PushSelect, None, cmd::ALL, "all"),
            kv(ModeId::PushSelect, None, cmd::NONE, "none"),
            kv(ModeId::PushSelect, None, cmd::PUSH, "push"),
            kv(ModeId::PushSelect, None, cmd::CANCEL, "cancel"),
        ]),
        ModeId::Conflicts => join_segments(&[
            format!(
                "{}/{}:nav",
                key_for_hint(ModeId::Conflicts, None, cmd::DOWN),
                key_for_hint(ModeId::Conflicts, None, cmd::UP)
            ),
            kv(ModeId::Conflicts, None, cmd::RESOLVE, "resolve"),
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

fn key_for_hint_any_pending(mode: ModeId, label: &str, format: KeyFormat) -> String {
    first_key_any_pending(mode, label, DisplayKind::Primary, format)
        .unwrap_or_else(|| "?".to_string())
}

fn kv(mode: ModeId, pending: Option<char>, label: &str, value: &str) -> String {
    format!("{}:{value}", key_for_hint(mode, pending, label))
}

fn join_segments(segments: &[String]) -> String {
    segments.join("  ")
}
