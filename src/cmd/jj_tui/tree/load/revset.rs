use crate::cmd::jj_tui::tree::TreeLoadScope;

pub(super) fn revset_for_scope(base: &str, load_scope: TreeLoadScope) -> String {
    match load_scope {
        TreeLoadScope::Stack => format!("{base} | ancestors(immutable_heads().., 2) | @::"),
        TreeLoadScope::Neighborhood => format!("{base} | ancestors(immutable_heads()..) | @::"),
    }
}
