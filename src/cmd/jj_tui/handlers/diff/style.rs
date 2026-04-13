use crate::cmd::jj_tui::state::{DiffLineKind, StyledSpan};
use ratatui::style::Color;
use syntect::highlighting::Style as SyntectStyle;
use syntect::parsing::SyntaxSet;

pub(super) fn syntax_for_file<'a>(
    current_file: Option<&str>,
    syntax_set: &'a SyntaxSet,
) -> Option<&'a syntect::parsing::SyntaxReference> {
    current_file.and_then(|file| {
        std::path::Path::new(file)
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| syntax_set.find_syntax_by_extension(ext))
    })
}

pub(super) fn code_prefix(kind: DiffLineKind) -> (&'static str, Color) {
    match kind {
        DiffLineKind::Added => ("+", Color::Green),
        DiffLineKind::Removed => ("-", Color::Red),
        DiffLineKind::Context => (" ", Color::DarkGray),
        DiffLineKind::FileHeader | DiffLineKind::Hunk => ("", Color::DarkGray),
    }
}

pub(super) fn header_spans(line: &str, kind: DiffLineKind) -> Vec<StyledSpan> {
    let color = match kind {
        DiffLineKind::FileHeader => Color::Yellow,
        DiffLineKind::Hunk => Color::Cyan,
        DiffLineKind::Added | DiffLineKind::Removed | DiffLineKind::Context => Color::White,
    };
    vec![StyledSpan {
        text: line.to_string(),
        fg: color,
    }]
}

pub(super) fn syntect_to_ratatui_color(style: SyntectStyle) -> Color {
    Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}
