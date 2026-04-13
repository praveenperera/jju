mod compile;
mod metadata;

use super::KeySequence;
use crate::cmd::jj_tui::keybindings::{ActionTemplate, Binding, ModeId};
use eyre::Result;

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
        compile::compile_bindings(self)
    }

    pub(crate) fn effective_prefix_title(&self) -> Option<&'static str> {
        metadata::effective_prefix_title(self)
    }
}
