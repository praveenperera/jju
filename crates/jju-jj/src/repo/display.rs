use super::{CommitDetails, JjRepo};
use eyre::Result;
use jj_lib::commit::Commit;
use jj_lib::id_prefix::IdPrefixIndex;

impl JjRepo {
    pub fn description_first_line(commit: &Commit) -> String {
        commit
            .description()
            .lines()
            .next()
            .unwrap_or("")
            .to_string()
    }

    pub fn author_name(commit: &Commit) -> String {
        commit.author().name.clone()
    }

    pub fn author_email(commit: &Commit) -> String {
        commit.author().email.clone()
    }

    pub fn author_timestamp_relative(commit: &Commit) -> String {
        let timestamp = commit.author().timestamp;
        let millis = timestamp.timestamp.0;
        let secs = millis / 1000;
        let Some(datetime) = chrono::DateTime::from_timestamp(secs, 0) else {
            return "unknown".to_string();
        };

        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(datetime);
        let absolute = datetime.format("%Y-%m-%d %H:%M");
        let relative = if diff.num_days() > 365 {
            format!("{} years ago", diff.num_days() / 365)
        } else if diff.num_days() > 30 {
            format!("{} months ago", diff.num_days() / 30)
        } else if diff.num_days() > 0 {
            format!("{} days ago", diff.num_days())
        } else if diff.num_hours() > 0 {
            format!("{} hours ago", diff.num_hours())
        } else if diff.num_minutes() > 0 {
            format!("{} minutes ago", diff.num_minutes())
        } else {
            "just now".to_string()
        };
        format!("{relative} ({absolute})")
    }

    pub fn commit_details_with_index(
        &self,
        commit: &Commit,
        index: &IdPrefixIndex,
    ) -> Result<CommitDetails> {
        let (_, unique_commit_prefix_len) = self.commit_id_with_index(index, commit, 7)?;

        Ok(CommitDetails {
            unique_commit_prefix_len,
            full_description: commit.description().to_string(),
            author_name: Self::author_name(commit),
            author_email: Self::author_email(commit),
            timestamp: Self::author_timestamp_relative(commit),
        })
    }
}
