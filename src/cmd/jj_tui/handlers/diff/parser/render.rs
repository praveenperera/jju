use super::super::style::code_prefix;
use super::highlight::highlighted_spans;
use crate::cmd::jj_tui::state::{DiffLineKind, StyledSpan};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub(super) fn code_spans(
    kind: DiffLineKind,
    code: &str,
    current_file: Option<&str>,
    syntax_set: &SyntaxSet,
    plain_text: &SyntaxReference,
    theme: &syntect::highlighting::Theme,
) -> Vec<StyledSpan> {
    let (prefix, prefix_color) = code_prefix(kind);
    let mut spans = vec![StyledSpan {
        text: prefix.to_string(),
        fg: prefix_color,
    }];
    spans.extend(highlighted_spans(
        code,
        current_file,
        syntax_set,
        plain_text,
        theme,
    ));
    spans
}
