use super::super::{App, DetailHydrationUpdate, DetailHydrator};
use crate::jj_lib_helpers::JjRepo;
use log::info;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Instant;

pub(super) fn start_detail_hydration(app: &mut App, commit_ids: Vec<String>) {
    if commit_ids.is_empty() {
        app.detail_hydrator = None;
        return;
    }

    app.detail_generation += 1;
    let generation = app.detail_generation;
    let repo_path = app.repo_path.clone();
    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move || {
        let started_at = Instant::now();
        let Ok(jj_repo) = JjRepo::load(Some(&repo_path)) else {
            return;
        };
        let Ok(hydrated_count) = jj_repo.with_short_prefix_index(|prefix_index| {
            let mut hydrated_count = 0;

            for commit_id in commit_ids {
                let Ok(commit) = jj_repo.commit_by_id_hex(&commit_id) else {
                    continue;
                };
                let Ok(details) = jj_repo.commit_details_with_index(&commit, prefix_index) else {
                    continue;
                };

                if sender
                    .send(DetailHydrationUpdate {
                        generation,
                        commit_id,
                        details,
                    })
                    .is_err()
                {
                    break;
                }

                hydrated_count += 1;
            }

            Ok(hydrated_count)
        }) else {
            return;
        };

        info!(
            "Hydrated {} tree rows in {:?}",
            hydrated_count,
            started_at.elapsed()
        );
    });

    app.detail_hydrator = Some(DetailHydrator {
        generation,
        receiver,
    });
}

pub(super) fn apply_detail_updates(app: &mut App) {
    let Some(hydrator) = &mut app.detail_hydrator else {
        return;
    };

    let generation = hydrator.generation;
    let mut updates = Vec::new();
    let mut disconnected = false;

    loop {
        match hydrator.receiver.try_recv() {
            Ok(update) => updates.push(update),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                disconnected = true;
                break;
            }
        }
    }

    for update in updates {
        if update.generation != generation {
            continue;
        }
        app.tree.hydrate_details(&update.commit_id, update.details);
    }

    if disconnected {
        app.detail_hydrator = None;
    }
}
