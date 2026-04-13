use super::{ActionTemplate, KeyPattern, ModeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    Primary,
    Alias,
}

#[derive(Debug)]
pub struct Binding {
    pub mode: ModeId,
    pub pending_prefix: Option<char>,
    pub key: KeyPattern,
    pub action: ActionTemplate,
    pub display: DisplayKind,
    pub label: &'static str,
}
