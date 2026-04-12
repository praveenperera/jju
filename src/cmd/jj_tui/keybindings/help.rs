use super::catalog;
use super::display::{
    KeyFormat, display_key_pattern, format_binding_key, join_keys, keys_for_label,
};

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

    for binding in super::bindings() {
        let Some((section, description)) = binding.help else {
            continue;
        };

        let keys = if binding.pending_prefix.is_some() {
            format_binding_key(binding, KeyFormat::Space)
        } else {
            display_key_pattern(&binding.key)
        };

        sections
            .entry(section)
            .or_default()
            .push(HelpItemView { keys, description });
    }

    for (mode, prefix, label, section) in catalog::HELP_ALIAS_ITEMS {
        let keys = keys_for_label(*mode, *prefix, label, true, KeyFormat::Space);
        if keys.len() > 1 {
            let joined = join_keys(&keys, "/");
            if let Some(items) = sections.get_mut(section) {
                for item in items.iter_mut() {
                    if item.keys == keys[0] {
                        item.keys = joined.clone();
                        break;
                    }
                }
            }
        }
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
