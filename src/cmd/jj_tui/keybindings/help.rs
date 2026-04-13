mod build;

use build::build_help_sections;

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
