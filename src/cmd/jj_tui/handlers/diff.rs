//! Diff parsing and syntax highlighting
//!
//! This module handles parsing git diffs and applying syntax highlighting.

use crate::cmd::jj_tui::state::{DiffLine, DiffLineKind, DiffStats, StyledSpan};
use ratatui::style::Color;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

/// Parse diff stats output from jj diff --stat
pub fn parse_diff_stats(output: &str) -> DiffStats {
    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for line in output.lines() {
        // look for the summary line
        if line.contains("file") && line.contains("changed") {
            // parse: "N file(s) changed, M insertion(s)(+), K deletion(s)(-)"
            for part in line.split(',') {
                let part = part.trim();
                if part.contains("file") {
                    if let Some(num) = part.split_whitespace().next() {
                        files_changed = num.parse().unwrap_or(0);
                    }
                } else if part.contains("insertion") {
                    if let Some(num) = part.split_whitespace().next() {
                        insertions = num.parse().unwrap_or(0);
                    }
                } else if part.contains("deletion") {
                    if let Some(num) = part.split_whitespace().next() {
                        deletions = num.parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    DiffStats {
        files_changed,
        insertions,
        deletions,
    }
}

fn syntect_to_ratatui_color(style: SyntectStyle) -> Color {
    Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

/// Parse diff output into styled lines with syntax highlighting
pub fn parse_diff(output: &str, ss: &SyntaxSet, ts: &ThemeSet) -> Vec<DiffLine> {
    let theme = &ts.themes["base16-eighties.dark"];
    let plain_text = ss.find_syntax_plain_text();

    let mut current_file: Option<String> = None;
    let mut lines = Vec::new();

    for line in output.lines() {
        let (kind, code_content) = if line.starts_with("diff --git") {
            // extract filename from "diff --git a/path/file.rs b/path/file.rs"
            if let Some(b_path) = line.split(" b/").nth(1) {
                current_file = Some(b_path.to_string());
            }
            (DiffLineKind::FileHeader, None)
        } else if line.starts_with("+++") || line.starts_with("---") {
            (DiffLineKind::FileHeader, None)
        } else if line.starts_with("@@") {
            (DiffLineKind::Hunk, None)
        } else if let Some(rest) = line.strip_prefix('+') {
            (DiffLineKind::Added, Some(rest))
        } else if let Some(rest) = line.strip_prefix('-') {
            (DiffLineKind::Removed, Some(rest))
        } else if let Some(rest) = line.strip_prefix(' ') {
            (DiffLineKind::Context, Some(rest))
        } else {
            (DiffLineKind::Context, Some(line))
        };

        let spans = if let Some(code) = code_content {
            let prefix = match kind {
                DiffLineKind::Added => "+",
                DiffLineKind::Removed => "-",
                DiffLineKind::Context => " ",
                _ => "",
            };

            let prefix_color = match kind {
                DiffLineKind::Added => Color::Green,
                DiffLineKind::Removed => Color::Red,
                _ => Color::DarkGray,
            };

            // try syntect highlighting
            let syntax = current_file.as_ref().and_then(|f| {
                std::path::Path::new(f)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| ss.find_syntax_by_extension(ext))
            });

            let code_spans = if let Some(syn) = syntax {
                let mut highlighter = syntect::easy::HighlightLines::new(syn, theme);
                highlighter.highlight_line(code, ss).ok().map(|ranges| {
                    ranges
                        .into_iter()
                        .map(|(style, text)| StyledSpan {
                            text: text.to_string(),
                            fg: syntect_to_ratatui_color(style),
                        })
                        .collect::<Vec<_>>()
                })
            } else {
                None
            };

            // fall back to plain text
            let code_spans = code_spans.unwrap_or_else(|| {
                let mut highlighter = syntect::easy::HighlightLines::new(plain_text, theme);
                highlighter
                    .highlight_line(code, ss)
                    .map(|ranges| {
                        ranges
                            .into_iter()
                            .map(|(style, text)| StyledSpan {
                                text: text.to_string(),
                                fg: syntect_to_ratatui_color(style),
                            })
                            .collect()
                    })
                    .unwrap_or_else(|_| {
                        vec![StyledSpan {
                            text: code.to_string(),
                            fg: Color::White,
                        }]
                    })
            });

            let mut result = vec![StyledSpan {
                text: prefix.to_string(),
                fg: prefix_color,
            }];
            result.extend(code_spans);
            result
        } else {
            // non-code lines (headers, hunks)
            let color = match kind {
                DiffLineKind::FileHeader => Color::Yellow,
                DiffLineKind::Hunk => Color::Cyan,
                _ => Color::White,
            };
            vec![StyledSpan {
                text: line.to_string(),
                fg: color,
            }]
        };

        lines.push(DiffLine { spans, kind });
    }

    lines
}
