use super::command::capture_stdout;
use eyre::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffOps;

impl DiffOps {
    pub fn get_diff(self, rev: &str) -> Result<String> {
        capture_stdout(&["diff", "--git", "-r", rev])
    }

    pub fn get_stats(self, change_id: &str) -> Result<String> {
        capture_stdout(&["diff", "--stat", "-r", change_id])
    }
}
