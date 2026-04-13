use crate::cmd::jj_tui::state::DiffStats;

/// Parse diff stats output from jj diff --stat
pub fn parse_diff_stats(output: &str) -> DiffStats {
    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for line in output.lines() {
        // look for the summary line
        if line.contains("file") && line.contains("changed") {
            for part in line.split(',') {
                let part = part.trim();
                if part.contains("file")
                    && let Some(num) = part.split_whitespace().next()
                {
                    files_changed = num.parse().unwrap_or(0);
                } else if part.contains("insertion")
                    && let Some(num) = part.split_whitespace().next()
                {
                    insertions = num.parse().unwrap_or(0);
                } else if part.contains("deletion")
                    && let Some(num) = part.split_whitespace().next()
                {
                    deletions = num.parse().unwrap_or(0);
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
