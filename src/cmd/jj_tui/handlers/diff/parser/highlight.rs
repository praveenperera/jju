use super::super::style::{syntax_for_file, syntect_to_ratatui_color};
use crate::cmd::jj_tui::state::StyledSpan;
use ratatui::style::Color;
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub(super) fn highlight_code(
    code: &str,
    syntax: &SyntaxReference,
    syntax_set: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Option<Vec<StyledSpan>> {
    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    highlighter
        .highlight_line(code, syntax_set)
        .ok()
        .map(|ranges| {
            ranges
                .into_iter()
                .map(|(style, text)| StyledSpan {
                    text: text.to_string(),
                    fg: syntect_to_ratatui_color(style),
                })
                .collect()
        })
}

pub(super) fn highlighted_spans(
    code: &str,
    current_file: Option<&str>,
    syntax_set: &SyntaxSet,
    plain_text: &SyntaxReference,
    theme: &syntect::highlighting::Theme,
) -> Vec<StyledSpan> {
    syntax_for_file(current_file, syntax_set)
        .and_then(|syntax| highlight_code(code, syntax, syntax_set, theme))
        .unwrap_or_else(|| {
            highlight_code(code, plain_text, syntax_set, theme).unwrap_or_else(|| {
                vec![StyledSpan {
                    text: code.to_string(),
                    fg: Color::White,
                }]
            })
        })
}
