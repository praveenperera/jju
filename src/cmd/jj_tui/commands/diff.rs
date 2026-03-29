use super::common::capture_stdout;
use eyre::Result;

pub fn get_diff(rev: &str) -> Result<String> {
    capture_stdout(&["diff", "--git", "-r", rev])
}

pub fn get_stats(change_id: &str) -> Result<String> {
    capture_stdout(&["diff", "--stat", "-r", change_id])
}
