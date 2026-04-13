use super::style::{code_prefix, header_spans, syntax_for_file, syntect_to_ratatui_color};
use crate::cmd::jj_tui::state::{DiffLine, DiffLineKind, StyledSpan};
use ratatui::style::Color;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub(super) fn parse_diff(output: &str, syntax_set: &SyntaxSet, themes: &ThemeSet) -> Vec<DiffLine> {
    let theme = &themes.themes["base16-eighties.dark"];
    let plain_text = syntax_set.find_syntax_plain_text();

    let mut current_file: Option<String> = None;
    let mut lines = Vec::new();

    for line in output.lines() {
        let (kind, code_content) = classify_diff_line(line, &mut current_file);
        let spans = if let Some(code) = code_content {
            code_spans(
                kind,
                code,
                current_file.as_deref(),
                syntax_set,
                plain_text,
                theme,
            )
        } else {
            header_spans(line, kind)
        };
        lines.push(DiffLine { spans, kind });
    }

    lines
}

fn classify_diff_line<'a>(
    line: &'a str,
    current_file: &mut Option<String>,
) -> (DiffLineKind, Option<&'a str>) {
    if line.starts_with("diff --git") {
        if let Some(b_path) = line.split(" b/").nth(1) {
            *current_file = Some(b_path.to_string());
        }
        return (DiffLineKind::FileHeader, None);
    }

    if line.starts_with("+++") || line.starts_with("---") {
        return (DiffLineKind::FileHeader, None);
    }

    if line.starts_with("@@") {
        return (DiffLineKind::Hunk, None);
    }

    if let Some(rest) = line.strip_prefix('+') {
        return (DiffLineKind::Added, Some(rest));
    }

    if let Some(rest) = line.strip_prefix('-') {
        return (DiffLineKind::Removed, Some(rest));
    }

    if let Some(rest) = line.strip_prefix(' ') {
        return (DiffLineKind::Context, Some(rest));
    }

    (DiffLineKind::Context, Some(line))
}

fn code_spans(
    kind: DiffLineKind,
    code: &str,
    current_file: Option<&str>,
    syntax_set: &SyntaxSet,
    plain_text: &syntect::parsing::SyntaxReference,
    theme: &syntect::highlighting::Theme,
) -> Vec<StyledSpan> {
    let (prefix, prefix_color) = code_prefix(kind);
    let syntax = syntax_for_file(current_file, syntax_set);

    let highlighted = syntax
        .and_then(|syntax| highlight_code(code, syntax, syntax_set, theme))
        .unwrap_or_else(|| {
            highlight_code(code, plain_text, syntax_set, theme).unwrap_or_else(|| {
                vec![StyledSpan {
                    text: code.to_string(),
                    fg: Color::White,
                }]
            })
        });

    let mut spans = vec![StyledSpan {
        text: prefix.to_string(),
        fg: prefix_color,
    }];
    spans.extend(highlighted);
    spans
}

fn highlight_code(
    code: &str,
    syntax: &syntect::parsing::SyntaxReference,
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
