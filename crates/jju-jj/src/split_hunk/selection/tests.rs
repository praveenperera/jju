use super::{SplitHunkPlanner, parse_hunk_indices, parse_line_ranges};
use crate::split_hunk::SplitHunkOptions;
use crate::split_hunk::diff::ParsedDiff;

fn options() -> SplitHunkOptions {
    SplitHunkOptions {
        message: None,
        revision: "@".to_string(),
        file_filter: None,
        lines: None,
        hunks: None,
        pattern: None,
        preview: false,
        invert: false,
        dry_run: false,
    }
}

fn parsed_diff() -> ParsedDiff {
    ParsedDiff::parse(
        r#"diff --git a/src/lib.rs b/src/lib.rs
@@ -1,1 +1,2 @@
 line one
+selected
@@ -10,1 +10,2 @@
 line ten
+other
"#,
    )
}

#[test]
fn test_parse_line_ranges_supports_ranges_and_single_lines() {
    let ranges = parse_line_ranges("3-5,8").expect("parse line ranges");
    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0].0, 3);
    assert_eq!(ranges[0].1, 5);
    assert_eq!(ranges[1].0, 8);
    assert_eq!(ranges[1].1, 8);
}

#[test]
fn test_parse_hunk_indices_supports_multiple_entries() {
    assert_eq!(
        parse_hunk_indices("0,2,5").expect("parse hunk indices"),
        vec![0, 2, 5]
    );
}

#[test]
fn test_build_selects_by_hunk_index() {
    let mut options = options();
    options.hunks = Some("1".to_string());

    let plan = SplitHunkPlanner::from_options(&options)
        .expect("planner")
        .build(parsed_diff());

    assert_eq!(plan.selected_count(), 1);
    assert_eq!(plan.selected_files()[0].selected_hunks, vec![1]);
}

#[test]
fn test_build_selects_by_pattern() {
    let mut options = options();
    options.pattern = Some("selected".to_string());

    let plan = SplitHunkPlanner::from_options(&options)
        .expect("planner")
        .build(parsed_diff());

    assert_eq!(plan.selected_count(), 1);
    assert_eq!(plan.selected_files()[0].selected_hunks, vec![0]);
}

#[test]
fn test_build_inverts_matches() {
    let mut options = options();
    options.pattern = Some("selected".to_string());
    options.invert = true;

    let plan = SplitHunkPlanner::from_options(&options)
        .expect("planner")
        .build(parsed_diff());

    assert_eq!(plan.selected_count(), 1);
    assert_eq!(plan.selected_files()[0].selected_hunks, vec![1]);
}
