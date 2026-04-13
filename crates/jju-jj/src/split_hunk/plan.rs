use super::diff::{FileDiff, ParsedDiff};
use super::selection::SelectedHunk;
use ahash::{HashMap, HashMapExt};

#[derive(Debug, Clone)]
pub(crate) struct FileSelection<'a> {
    pub(crate) file: &'a FileDiff,
    pub(crate) selected_hunks: Vec<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct SplitHunkPlan {
    diff: ParsedDiff,
    selected: Vec<SelectedHunk>,
}

impl SplitHunkPlan {
    pub(crate) fn new(diff: ParsedDiff, selected: Vec<SelectedHunk>) -> Self {
        Self { diff, selected }
    }

    pub(crate) fn files(&self) -> &[FileDiff] {
        self.diff.files()
    }

    pub(crate) fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub(crate) fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }

    pub(crate) fn selected_files(&self) -> Vec<FileSelection<'_>> {
        let mut grouped = HashMap::<usize, Vec<usize>>::new();
        for selected in &self.selected {
            grouped
                .entry(selected.file_index)
                .or_default()
                .push(selected.hunk_index);
        }

        let mut entries = grouped.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(file_index, _)| *file_index);

        entries
            .into_iter()
            .map(|(file_index, selected_hunks)| FileSelection {
                file: &self.diff.files()[file_index],
                selected_hunks,
            })
            .collect()
    }
}
