use super::App;
use crate::cmd::jj_tui::handlers;
use crate::cmd::jj_tui::state::DiffStats;
use eyre::Result;

impl App {
    pub fn get_diff_stats(&mut self, change_id: &str) -> Option<&DiffStats> {
        if !self.diff_stats_cache.contains_key(change_id)
            && let Ok(stats) = self.fetch_diff_stats(change_id)
        {
            self.diff_stats_cache.insert(change_id.to_string(), stats);
        }
        self.diff_stats_cache.get(change_id)
    }

    fn fetch_diff_stats(&self, change_id: &str) -> Result<DiffStats> {
        let output = crate::cmd::jj_tui::commands::diff::get_stats(change_id)?;
        Ok(handlers::diff::parse_diff_stats(&output))
    }
}
