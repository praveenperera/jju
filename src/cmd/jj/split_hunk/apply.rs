use super::{DiffHunk, FileDiff, read_file_lines_or_empty};
use ahash::HashMap;
use eyre::Result;

pub(super) fn apply_selected_hunks(
    files: &[FileDiff],
    selected: &[(usize, usize)],
    revision: &str,
) -> Result<HashMap<String, String>> {
    let mut results = HashMap::default();
    let mut file_hunks: HashMap<usize, Vec<usize>> = HashMap::default();

    for &(file_idx, hunk_idx) in selected {
        file_hunks.entry(file_idx).or_default().push(hunk_idx);
    }

    for (file_idx, hunk_indices) in file_hunks {
        let file = &files[file_idx];
        let parent_rev = format!("{}-", revision);
        let parent_lines = read_file_lines_or_empty(&parent_rev, &file.path);
        let result_lines = apply_hunks_to_lines(&parent_lines, &file.hunks, &hunk_indices);
        results.insert(file.path.clone(), result_lines.join("\n"));
    }

    Ok(results)
}

pub(super) fn apply_hunks_to_lines(
    parent_lines: &[String],
    hunks: &[DiffHunk],
    selected_indices: &[usize],
) -> Vec<String> {
    let mut result_lines = parent_lines.to_vec();
    let mut sorted_indices = selected_indices.to_vec();
    sorted_indices.sort_by(|left, right| right.cmp(left));

    for hunk_idx in sorted_indices {
        let hunk = &hunks[hunk_idx];
        let insert_pos = hunk.old_start.saturating_sub(1);
        let remove_end = (insert_pos + hunk.old_count).min(result_lines.len());
        result_lines.drain(insert_pos..remove_end);

        let new_lines: Vec<String> = hunk
            .lines
            .iter()
            .filter(|line| !line.kind.is_removed())
            .map(|line| line.content.clone())
            .collect();

        for (idx, line) in new_lines.into_iter().enumerate() {
            result_lines.insert(insert_pos + idx, line);
        }
    }

    result_lines
}
