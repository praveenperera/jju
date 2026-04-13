use super::KeySequence;
use crate::cmd::jj_tui::{
    action::Action,
    keybindings::{ActionTemplate, Binding, DisplayKind, ModeId},
};
use eyre::{Result, eyre};

#[derive(Debug, Clone)]
pub enum BindingBehavior {
    Action(ActionTemplate),
    PendingPrefix { title: &'static str },
}

#[derive(Debug, Clone)]
pub struct HelpMetadata {
    pub section: &'static str,
    pub description: &'static str,
    pub include_aliases: bool,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub mode: ModeId,
    pub label: &'static str,
    pub behavior: BindingBehavior,
    pub help: Option<HelpMetadata>,
    pub prefix_title: Option<&'static str>,
    pub keys: Vec<KeySequence>,
}

impl CommandSpec {
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
        self.help = Some(HelpMetadata {
            section,
            description,
            include_aliases: false,
        });
        self
    }

    pub(crate) fn help_aliases(mut self) -> Self {
        if let Some(help) = &mut self.help {
            help.include_aliases = true;
        }
        self
    }

    pub(crate) fn prefix_title(mut self, title: &'static str) -> Self {
        self.prefix_title = Some(title);
        self
    }

    pub(crate) fn uses_prefix(&self, prefix: char) -> bool {
        self.keys.iter().any(|key| key.prefix() == Some(prefix))
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
