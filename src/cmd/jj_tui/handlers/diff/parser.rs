mod classify;
mod highlight;
mod render;

use super::style::header_spans;
use crate::cmd::jj_tui::state::DiffLine;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub(super) fn parse_diff(output: &str, syntax_set: &SyntaxSet, themes: &ThemeSet) -> Vec<DiffLine> {
    let theme = &themes.themes["base16-eighties.dark"];
    let plain_text = syntax_set.find_syntax_plain_text();

    let mut current_file: Option<String> = None;
    let mut lines = Vec::new();

    for line in output.lines() {
        let (kind, code_content) = classify::classify_diff_line(line, &mut current_file);
        let spans = if let Some(code) = code_content {
            render::code_spans(
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
