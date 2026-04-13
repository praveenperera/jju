use super::display::{KeyFormat, display_keys_for_command, join_keys};

const HELP_SECTION_ORDER: &[&str] = &[
    "Navigation",
    "View",
    "Edit Operations",
    "Selection",
    "Rebase",
    "Bookmarks & Git",
    "Conflicts",
    "General",
];

#[derive(Debug, Clone)]
pub struct HelpItemView {
    pub keys: String,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct HelpSectionView {
    pub title: &'static str,
    pub items: Vec<HelpItemView>,
}

pub fn build_help_view() -> Vec<HelpSectionView> {
    use std::collections::HashMap;

    let mut sections: HashMap<&'static str, Vec<HelpItemView>> = HashMap::new();

    for command in super::commands() {
        let Some(help) = &command.help else {
            continue;
        };

        let keys = display_keys_for_command(
            command.mode,
            command.label,
            help.include_aliases,
            KeyFormat::Space,
        );
        if keys.is_empty() {
            continue;
        }
        let keys = if help.include_aliases {
            join_keys(&keys, "/")
        } else {
            keys[0].clone()
        };

        sections
            .entry(help.section)
            .or_default()
            .push(HelpItemView {
                keys,
                description: help.description,
            });
    }

    let mut out = Vec::new();
    for &title in HELP_SECTION_ORDER {
        if let Some(items) = sections.remove(title) {
            out.push(HelpSectionView { title, items });
        }
    }

    for (title, items) in sections {
        out.push(HelpSectionView { title, items });
    }

    out
}
