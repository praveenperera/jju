use super::model::{DiffHunk, DiffLine, FileDiff, ParsedDiff};

impl ParsedDiff {
    pub(crate) fn parse(diff_output: &str) -> Self {
        let mut files = Vec::new();
        let mut current_file: Option<FileDiff> = None;
        let mut current_hunk: Option<DiffHunk> = None;

        for line in diff_output.lines() {
            if line.starts_with("diff --git ") {
                flush_current_hunk(&mut current_file, &mut current_hunk);
                flush_current_file(&mut files, &mut current_file);
                current_file = Some(FileDiff::new(parse_file_path(line)));
                continue;
            }

            if line.starts_with("@@ ") {
                flush_current_hunk(&mut current_file, &mut current_hunk);
                current_hunk = Some(DiffHunk::parse_header(line));
                continue;
            }

            if let Some(hunk) = &mut current_hunk
                && let Some(diff_line) = DiffLine::parse(line)
            {
                hunk.push_line(diff_line);
            }
        }

        flush_current_hunk(&mut current_file, &mut current_hunk);
        flush_current_file(&mut files, &mut current_file);

        Self::from_files(files)
    }
}

fn parse_file_path(line: &str) -> String {
    line.split_whitespace()
        .nth(3)
        .map(|part| part.trim_start_matches("b/"))
        .unwrap_or("")
        .to_string()
}

fn flush_current_hunk(current_file: &mut Option<FileDiff>, current_hunk: &mut Option<DiffHunk>) {
    if let Some(hunk) = current_hunk.take()
        && let Some(file) = current_file
    {
        file.push_hunk(hunk);
    }
}

fn flush_current_file(files: &mut Vec<FileDiff>, current_file: &mut Option<FileDiff>) {
    if let Some(file) = current_file.take() {
        files.push(file);
    }
}
