mod header;
mod model;
mod parse;
#[cfg(test)]
mod tests;

pub(crate) use model::{DiffHunk, DiffLineKind, FileDiff, ParsedDiff};
