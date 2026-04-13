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

pub fn build_help_view() -> Vec<HelpSectionView> {
    let mut sections = build_help_sections();
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

fn build_help_sections() -> std::collections::HashMap<&'static str, Vec<HelpItemView>> {
    let mut sections = std::collections::HashMap::new();

    for command in crate::cmd::jj_tui::keybindings::commands() {
        let Some(help) = &command.help else {
            continue;
        };

        let keys = crate::cmd::jj_tui::keybindings::display::display_keys_for_command(
            command.mode,
            command.label,
            help.include_aliases,
            crate::cmd::jj_tui::keybindings::display::KeyFormat::Space,
        );
        if keys.is_empty() {
            continue;
        }
        let keys = if help.include_aliases {
            crate::cmd::jj_tui::keybindings::display::join_keys(&keys, "/")
        } else {
            keys[0].clone()
        };

        sections
            .entry(help.section)
            .or_insert_with(Vec::new)
            .push(HelpItemView {
                keys,
                description: help.description,
            });
    }

    sections
}
