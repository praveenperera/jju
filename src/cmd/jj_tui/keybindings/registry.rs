use super::{Binding, BindingDef, bindings as binding_defs};
use std::sync::OnceLock;

static BINDINGS: OnceLock<Vec<Binding>> = OnceLock::new();

pub(crate) fn bindings() -> &'static [Binding] {
    BINDINGS.get_or_init(|| {
        binding_defs::binding_defs()
            .iter()
            .map(BindingDef::to_binding)
            .collect()
    })
}

pub fn prefix_title(prefix: char) -> Option<&'static str> {
    binding_defs::prefix_title(prefix)
}

pub fn is_known_prefix(prefix: char) -> bool {
    prefix_title(prefix).is_some()
}
