use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyPattern {
    Exact {
        code: KeyCode,
        required_mods: KeyModifiers,
    },
    AnyChar,
}

impl KeyPattern {
    pub(crate) fn matches(&self, event: &KeyEvent) -> Option<MatchCapture> {
        match self {
            KeyPattern::Exact {
                code,
                required_mods,
            } => {
                if &event.code == code && event.modifiers.contains(*required_mods) {
                    Some(MatchCapture::None)
                } else {
                    None
                }
            }
            KeyPattern::AnyChar => match event.code {
                KeyCode::Char(c) => Some(MatchCapture::Char(c)),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatchCapture {
    None,
    Char(char),
}

impl MatchCapture {
    pub(crate) fn char(self) -> Option<char> {
        match self {
            MatchCapture::None => None,
            MatchCapture::Char(c) => Some(c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyDef {
    Char(char),
    Ctrl(char),
    Key(KeyCode),
    AnyChar,
}

impl KeyDef {
    pub(crate) const fn to_pattern(self) -> KeyPattern {
        match self {
            KeyDef::Char(c) => KeyPattern::Exact {
                code: KeyCode::Char(c),
                required_mods: KeyModifiers::NONE,
            },
            KeyDef::Ctrl(c) => KeyPattern::Exact {
                code: KeyCode::Char(c),
                required_mods: KeyModifiers::CONTROL,
            },
            KeyDef::Key(code) => KeyPattern::Exact {
                code,
                required_mods: KeyModifiers::NONE,
            },
            KeyDef::AnyChar => KeyPattern::AnyChar,
        }
    }

    pub(crate) const fn plain_char(self) -> Option<char> {
        match self {
            KeyDef::Char(c) => Some(c),
            KeyDef::Ctrl(_) | KeyDef::Key(_) | KeyDef::AnyChar => None,
        }
    }
}
