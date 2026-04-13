use crate::cmd::jj_tui::keybindings::{KeyDef, KeyPattern};
use eyre::{Result, eyre};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeySequence {
    Single(KeyDef),
    Chord(char, KeyDef),
}

impl KeySequence {
    pub(crate) fn compile(self) -> Result<(Option<char>, KeyPattern)> {
        match self {
            KeySequence::Single(key) => Ok((None, key.to_pattern())),
            KeySequence::Chord(prefix, key) => {
                if matches!(key, KeyDef::AnyChar) {
                    return Err(eyre!("`AnyChar` is only supported for single-key bindings"));
                }
                Ok((Some(prefix), key.to_pattern()))
            }
        }
    }

    pub(crate) const fn prefix(self) -> Option<char> {
        match self {
            KeySequence::Single(_) => None,
            KeySequence::Chord(prefix, _) => Some(prefix),
        }
    }

    pub(crate) fn pending_char(self) -> Option<char> {
        match self {
            KeySequence::Single(key) => key.plain_char(),
            KeySequence::Chord(_, _) => None,
        }
    }
}
