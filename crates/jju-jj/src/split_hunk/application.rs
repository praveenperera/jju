use super::plan::SplitHunkPlan;
use super::repo::SplitHunkRepo;
use ahash::HashMap;
use eyre::Result;

#[derive(Debug, Clone)]
pub(crate) struct SplitHunkApplication {
    pub(crate) new_contents: HashMap<String, String>,
    pub(crate) original_contents: HashMap<String, String>,
}

impl SplitHunkApplication {
    pub(crate) fn build(
        plan: &SplitHunkPlan,
        repo: &SplitHunkRepo,
        revision: &str,
    ) -> Result<Self> {
        let mut new_contents = HashMap::default();
        let parent_revision = format!("{revision}-");

        for selection in plan.selected_files() {
            let parent_lines =
                repo.read_file_lines_or_empty(&parent_revision, selection.file.path());
            let result_lines = apply_hunks_to_lines(
                &parent_lines,
                selection.file.hunks(),
                &selection.selected_hunks,
            );
            new_contents.insert(selection.file.path().to_string(), result_lines.join("\n"));
        }

        let original_contents = repo.collect_original_contents(plan.files(), revision);
        Ok(Self {
            new_contents,
            original_contents,
        })
    }
}

fn apply_hunks_to_lines(
    parent_lines: &[String],
    hunks: &[super::diff::DiffHunk],
    selected_indices: &[usize],
) -> Vec<String> {
    let mut result_lines = parent_lines.to_vec();
    let mut sorted_indices = selected_indices.to_vec();
    sorted_indices.sort_by(|left, right| right.cmp(left));

    for hunk_index in sorted_indices {
        let hunk = &hunks[hunk_index];
        let insert_pos = hunk.old_start().saturating_sub(1);
        let remove_end = (insert_pos + hunk.old_count()).min(result_lines.len());
        result_lines.drain(insert_pos..remove_end);

        let new_lines = hunk
            .lines()
            .iter()
            .filter(|line| !line.kind.is_removed())
            .map(|line| line.content.clone())
            .collect::<Vec<_>>();

        for (index, line) in new_lines.into_iter().enumerate() {
            result_lines.insert(insert_pos + index, line);
        }
    }

    result_lines
}

#[cfg(test)]
mod tests {
    use super::apply_hunks_to_lines;
    use crate::split_hunk::diff::ParsedDiff;

    #[test]
    fn test_apply_hunks_to_lines_applies_selected_hunks_bottom_up() {
        let diff = ParsedDiff::parse(
            r#"diff --git a/src/lib.rs b/src/lib.rs
@@ -1,1 +1,2 @@
 line one
+inserted one
@@ -3,1 +3,2 @@
 line three
+inserted three
"#,
        );
        let file = &diff.files()[0];
        let parent_lines = vec![
            "line one".to_string(),
            "line two".to_string(),
            "line three".to_string(),
        ];

        let applied = apply_hunks_to_lines(&parent_lines, file.hunks(), &[0, 1]);

        assert_eq!(
            applied,
            vec![
                "line one".to_string(),
                "inserted one".to_string(),
                "line two".to_string(),
                "line three".to_string(),
                "inserted three".to_string(),
            ]
        );
    }
}
