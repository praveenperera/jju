use super::App;
use super::replaceable_task::{CancellationToken, ReplaceableTask};
use crate::cmd::jj_tui::handlers;
use crate::cmd::jj_tui::state::DiffStats;
use crate::jj_lib_helpers::{CommitDetails, JjRepo};
use eyre::{Result, bail};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Sender, TryRecvError};
use std::time::{Duration, Instant};

const ROW_DATA_DEBOUNCE: Duration = Duration::from_millis(80);
const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RowDataRequest {
    visible_index: usize,
    commit_id: String,
    change_id: String,
    load_details: bool,
    load_stats: bool,
}

#[derive(Debug)]
pub(crate) struct RowDataUpdate {
    generation: u64,
    commit_id: String,
    details: Option<CommitDetails>,
    change_id: String,
    stats: Option<DiffStats>,
}

#[derive(Debug)]
pub(crate) enum RowDataLoader {
    Idle {
        generation: u64,
    },
    Pending {
        generation: u64,
        request: RowDataRequest,
        task: ReplaceableTask<RowDataUpdate>,
    },
}

impl Default for RowDataLoader {
    fn default() -> Self {
        Self::Idle { generation: 0 }
    }
}

impl RowDataLoader {
    fn generation(&self) -> u64 {
        match self {
            Self::Idle { generation } | Self::Pending { generation, .. } => *generation,
        }
    }

    fn request(&self) -> Option<&RowDataRequest> {
        match self {
            Self::Idle { .. } => None,
            Self::Pending { request, .. } => Some(request),
        }
    }

    fn next_generation(&self) -> u64 {
        self.generation() + 1
    }
}

impl App {
    pub(super) fn reset_row_data_loader(&mut self) {
        self.row_data_loader = RowDataLoader::Idle {
            generation: self.row_data_loader.next_generation(),
        };
    }

    pub(super) fn schedule_current_row_data_load(&mut self) {
        let Some(request) = self.current_row_data_request() else {
            self.row_data_loader = RowDataLoader::Idle {
                generation: self.row_data_loader.generation(),
            };
            return;
        };

        if self.row_data_loader.request() == Some(&request) {
            return;
        }

        let generation = self.row_data_loader.next_generation();
        let repo_path = self.repo_path.clone();
        let task_request = request.clone();

        self.row_data_loader = RowDataLoader::Pending {
            generation,
            request,
            task: ReplaceableTask::spawn(ROW_DATA_DEBOUNCE, move |token, sender| {
                load_row_data(token, sender, generation, repo_path, task_request);
            }),
        };
    }

    pub(super) fn apply_row_data_updates(&mut self) {
        let loader = std::mem::take(&mut self.row_data_loader);
        let RowDataLoader::Pending {
            generation,
            request,
            task,
        } = loader
        else {
            self.row_data_loader = loader;
            return;
        };

        let mut updates = Vec::new();
        let mut disconnected = false;

        loop {
            match task.receiver().try_recv() {
                Ok(update) => updates.push(update),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        if !disconnected {
            self.row_data_loader = RowDataLoader::Pending {
                generation,
                request,
                task,
            };
        } else {
            self.row_data_loader = RowDataLoader::Idle { generation };
        }

        for update in updates {
            if update.generation != generation {
                continue;
            }
            if let Some(details) = update.details {
                self.tree.hydrate_details(&update.commit_id, details);
            }
            if let Some(stats) = update.stats {
                self.diff_stats_cache.insert(update.change_id, stats);
            }
        }
    }

    fn current_row_data_request(&self) -> Option<RowDataRequest> {
        let entry = self.tree.current_entry()?;
        let node = &self.tree.nodes()[entry.node_index];
        let load_details = self.tree.is_expanded(self.tree.view.cursor) && !node.has_details();
        let load_stats = !self.diff_stats_cache.contains_key(&node.change_id);

        if !load_details && !load_stats {
            return None;
        }

        Some(RowDataRequest {
            visible_index: self.tree.view.cursor,
            commit_id: node.commit_id.clone(),
            change_id: node.change_id.clone(),
            load_details,
            load_stats,
        })
    }
}

fn load_row_data(
    token: CancellationToken,
    sender: Sender<RowDataUpdate>,
    generation: u64,
    repo_path: PathBuf,
    request: RowDataRequest,
) {
    let details = if request.load_details {
        load_details(&token, &repo_path, &request.commit_id)
    } else {
        None
    };

    if token.is_cancelled() {
        return;
    }

    let stats = if request.load_stats {
        fetch_diff_stats(&token, &repo_path, &request.change_id).ok()
    } else {
        None
    };

    if token.is_cancelled() {
        return;
    }

    let _ = sender.send(RowDataUpdate {
        generation,
        commit_id: request.commit_id,
        details,
        change_id: request.change_id,
        stats,
    });
}

fn load_details(
    token: &CancellationToken,
    repo_path: &Path,
    commit_id: &str,
) -> Option<CommitDetails> {
    if token.is_cancelled() {
        return None;
    }

    let jj_repo = JjRepo::load(Some(repo_path)).ok()?;
    if token.is_cancelled() {
        return None;
    }

    let commit = jj_repo.commit_by_id_hex(commit_id).ok()?;
    if token.is_cancelled() {
        return None;
    }

    jj_repo
        .with_short_prefix_index(|prefix_index| {
            jj_repo.commit_details_with_index(&commit, prefix_index)
        })
        .ok()
}

fn fetch_diff_stats(
    token: &CancellationToken,
    repo_path: &Path,
    change_id: &str,
) -> Result<DiffStats> {
    let output = capture_cancellable_diff_stats(token, repo_path, change_id)?;
    Ok(handlers::diff::parse_diff_stats(&output))
}

fn capture_cancellable_diff_stats(
    token: &CancellationToken,
    repo_path: &Path,
    change_id: &str,
) -> Result<String> {
    let mut child = diff_stats_command(repo_path, change_id).spawn()?;

    let started_at = Instant::now();

    loop {
        if token.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            bail!("diff stats cancelled");
        }

        if let Some(status) = child.try_wait()? {
            let mut output = String::new();
            if let Some(mut stdout) = child.stdout.take() {
                stdout.read_to_string(&mut output)?;
            }

            if status.success() {
                return Ok(output);
            }

            bail!("diff stats failed with exit code {:?}", status.code());
        }

        if started_at.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            bail!("diff stats timed out");
        }

        std::thread::sleep(CHILD_POLL_INTERVAL);
    }
}

fn diff_stats_command(repo_path: &Path, change_id: &str) -> Command {
    let mut command = Command::new("jj");
    command
        .current_dir(repo_path)
        .args(["diff", "--stat", "-r", change_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command
}

#[cfg(test)]
mod tests {
    use super::RowDataUpdate;
    use crate::cmd::jj_tui::state::DiffStats;
    use crate::cmd::jj_tui::test_support::{TestNodeKind, make_app_with_tree, make_tree};
    use crate::jj_lib_helpers::CommitDetails;

    #[test]
    fn matching_update_hydrates_details_and_stats() {
        let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
        let mut app = make_app_with_tree(tree);

        let update = RowDataUpdate {
            generation: 3,
            commit_id: "aaaa000000".to_string(),
            details: Some(CommitDetails {
                unique_commit_prefix_len: 6,
                full_description: "body".to_string(),
                author_name: "Praveen".to_string(),
                author_email: String::new(),
                timestamp: "today".to_string(),
            }),
            change_id: "aaaa".to_string(),
            stats: Some(DiffStats {
                files_changed: 1,
                insertions: 2,
                deletions: 3,
            }),
        };

        app.row_data_loader = super::RowDataLoader::Pending {
            generation: 3,
            request: super::RowDataRequest {
                visible_index: 0,
                commit_id: "aaaa000000".to_string(),
                change_id: "aaaa".to_string(),
                load_details: true,
                load_stats: true,
            },
            task: super::ReplaceableTask::spawn(
                std::time::Duration::from_millis(0),
                move |_token, sender| {
                    let _ = sender.send(update);
                },
            ),
        };

        std::thread::sleep(std::time::Duration::from_millis(20));
        app.apply_row_data_updates();

        assert!(app.tree.nodes()[0].details.is_some());
        assert!(app.diff_stats_cache.contains_key("aaaa"));
    }

    #[test]
    fn stale_update_is_ignored() {
        let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
        let mut app = make_app_with_tree(tree);

        let update = RowDataUpdate {
            generation: 3,
            commit_id: "aaaa000000".to_string(),
            details: None,
            change_id: "aaaa".to_string(),
            stats: Some(DiffStats {
                files_changed: 1,
                insertions: 2,
                deletions: 3,
            }),
        };

        app.row_data_loader = super::RowDataLoader::Pending {
            generation: 4,
            request: super::RowDataRequest {
                visible_index: 0,
                commit_id: "aaaa000000".to_string(),
                change_id: "aaaa".to_string(),
                load_details: false,
                load_stats: true,
            },
            task: super::ReplaceableTask::spawn(
                std::time::Duration::from_millis(0),
                move |_token, sender| {
                    let _ = sender.send(update);
                },
            ),
        };

        std::thread::sleep(std::time::Duration::from_millis(20));
        app.apply_row_data_updates();

        assert!(!app.diff_stats_cache.contains_key("aaaa"));
    }

    #[test]
    fn scheduling_does_not_block_or_fill_cache() {
        let tree = make_tree(vec![TestNodeKind::Plain.make_node("aaaa", 0)]);
        let mut app = make_app_with_tree(tree);

        app.schedule_current_row_data_load();

        assert!(app.diff_stats_cache.is_empty());
        assert!(matches!(
            app.row_data_loader,
            super::RowDataLoader::Pending { .. }
        ));
    }

    #[test]
    fn diff_stats_command_uses_repo_path() {
        let repo_path = std::path::Path::new("/tmp/jju-row-data-test");
        let command = super::diff_stats_command(repo_path, "abcd");

        assert_eq!(command.get_current_dir(), Some(repo_path));
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            ["diff", "--stat", "-r", "abcd"]
        );
    }
}
