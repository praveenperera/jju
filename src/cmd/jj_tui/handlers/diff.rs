//! Diff parsing and syntax highlighting
//!
//! This module handles parsing git diffs and applying syntax highlighting

mod parser;
mod stats;
mod style;

use crate::cmd::jj_tui::state::DiffLine;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub use stats::parse_diff_stats;

/// Parse diff output into styled lines with syntax highlighting
pub fn parse_diff(output: &str, ss: &SyntaxSet, ts: &ThemeSet) -> Vec<DiffLine> {
    parser::parse_diff(output, ss, ts)
}
