use super::{DiffLineKind, ParsedDiff};

#[test]
fn test_parse_diff_output_groups_files_and_hunks() {
    let parsed = ParsedDiff::parse(
        r#"diff --git a/src/lib.rs b/src/lib.rs
@@ -1,2 +1,3 @@
 line one
+line two
-line three
diff --git a/src/main.rs b/src/main.rs
@@ -3 +3 @@
-before
+after
"#,
    );

    assert_eq!(parsed.files().len(), 2);
    assert_eq!(parsed.files()[0].path(), "src/lib.rs");
    assert_eq!(parsed.files()[0].hunks().len(), 1);
    assert_eq!(parsed.files()[0].hunks()[0].first_line(), 1);
    assert_eq!(
        parsed.files()[0].hunks()[0].lines()[1].kind,
        DiffLineKind::Added
    );
    assert_eq!(parsed.files()[1].path(), "src/main.rs");
}
