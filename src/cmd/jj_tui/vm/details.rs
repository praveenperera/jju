use super::super::state::DiffStats;
use super::super::tree::TreeNode;

#[derive(Debug, Clone)]
pub struct RowDetails {
    pub commit_id_prefix: String,
    pub commit_id_suffix: String,
    pub author: String,
    pub timestamp: String,
    pub full_description: String,
    pub diff_stats: Option<DiffStats>,
}

pub(super) fn build_row_details(node: &TreeNode, stats: Option<&DiffStats>) -> RowDetails {
    let Some(details) = node.details.as_ref() else {
        let split_at = node.commit_id.len().min(12);
        let (commit_id_prefix, commit_id_suffix) = node.commit_id.split_at(split_at);

        return RowDetails {
            commit_id_prefix: commit_id_prefix.to_string(),
            commit_id_suffix: commit_id_suffix.to_string(),
            author: "loading...".to_string(),
            timestamp: "loading...".to_string(),
            full_description: "loading...".to_string(),
            diff_stats: stats.cloned(),
        };
    };

    let author = if details.author_email.is_empty() {
        details.author_name.clone()
    } else {
        format!("{} <{}>", details.author_name, details.author_email)
    };

    let (commit_prefix, commit_suffix) = node
        .commit_id
        .split_at(details.unique_commit_prefix_len.min(node.commit_id.len()));

    RowDetails {
        commit_id_prefix: commit_prefix.to_string(),
        commit_id_suffix: commit_suffix.to_string(),
        author,
        timestamp: details.timestamp.clone(),
        full_description: details.full_description.clone(),
        diff_stats: stats.cloned(),
    }
}

pub(super) fn row_height(details: Option<&RowDetails>) -> usize {
    match details {
        None => 1,
        Some(details) => {
            let desc_lines = details.full_description.trim().lines().count().max(1);
            1 + 5 + 1 + desc_lines
        }
    }
}
