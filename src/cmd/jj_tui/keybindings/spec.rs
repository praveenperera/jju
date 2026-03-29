use super::super::action::Action;
use super::{ActionTemplate, Binding, DisplayKind, KeyDef, KeyPattern, ModeId};
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

    fn pending_char(self) -> Option<char> {
        match self {
            KeySequence::Single(key) => key.plain_char(),
            KeySequence::Chord(_, _) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BindingBehavior {
    Action(ActionTemplate),
    PendingPrefix { title: &'static str },
}

#[derive(Debug, Clone)]
pub struct BindingSpec {
    pub mode: ModeId,
    pub label: &'static str,
    pub behavior: BindingBehavior,
    pub help: Option<(&'static str, &'static str)>,
    pub prefix_title: Option<&'static str>,
    pub keys: Vec<KeySequence>,
}

impl BindingSpec {
    pub(crate) fn new(
        mode: ModeId,
        label: &'static str,
        behavior: BindingBehavior,
        keys: Vec<KeySequence>,
    ) -> Self {
        Self {
            mode,
            label,
            behavior,
            help: None,
            prefix_title: None,
            keys,
        }
    }

    pub(crate) fn help(mut self, section: &'static str, description: &'static str) -> Self {
        self.help = Some((section, description));
        self
    }

    pub(crate) fn prefix_title(mut self, title: &'static str) -> Self {
        self.prefix_title = Some(title);
        self
    }

    pub(crate) fn compile_bindings(&self) -> Result<Vec<Binding>> {
        if self.keys.is_empty() {
            return Err(eyre!(
                "binding `{}` in mode {:?} must define at least one key",
                self.label,
                self.mode
            ));
        }

        let mut bindings = Vec::with_capacity(self.keys.len());
        for (index, key) in self.keys.iter().copied().enumerate() {
            let (pending_prefix, pattern) = key.compile()?;
            bindings.push(Binding {
                mode: self.mode,
                pending_prefix,
                key: pattern,
                action: self.compile_action(key)?,
                display: if index == 0 {
                    DisplayKind::Primary
                } else {
                    DisplayKind::Alias
                },
                label: self.label,
                help: if index == 0 { self.help } else { None },
            });
        }
        Ok(bindings)
    }

    pub(crate) fn effective_prefix_title(&self) -> Option<&'static str> {
        match self.behavior {
            BindingBehavior::PendingPrefix { title } => Some(title),
            BindingBehavior::Action(_) => self.prefix_title,
        }
    }

    fn compile_action(&self, key: KeySequence) -> Result<ActionTemplate> {
        match &self.behavior {
            BindingBehavior::Action(template) => Ok(template.clone()),
            BindingBehavior::PendingPrefix { .. } => key
                .pending_char()
                .map(|prefix| ActionTemplate::Fixed(Action::SetPendingKey(prefix)))
                .ok_or_else(|| {
                    eyre!(
                        "pending-prefix binding `{}` in mode {:?} must use plain character keys",
                        self.label,
                        self.mode
                    )
                }),
        }
    }
}
