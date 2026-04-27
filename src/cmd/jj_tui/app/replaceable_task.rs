use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub(crate) struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub(crate) struct ReplaceableTask<T> {
    cancel_token: Option<CancellationToken>,
    receiver: Receiver<T>,
}

impl<T> ReplaceableTask<T> {
    pub(crate) fn receiver(&self) -> &Receiver<T> {
        &self.receiver
    }

    pub(crate) fn cancel(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
    }
}

impl<T> ReplaceableTask<T>
where
    T: Send + 'static,
{
    pub(crate) fn spawn<F>(debounce: Duration, run: F) -> Self
    where
        F: FnOnce(CancellationToken, Sender<T>) + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let cancel_token = CancellationToken(Arc::new(AtomicBool::new(false)));
        let task_token = cancel_token.clone();

        std::thread::spawn(move || {
            let mut remaining = debounce;
            let step = Duration::from_millis(10);

            while !remaining.is_zero() {
                if task_token.is_cancelled() {
                    return;
                }

                let sleep_for = remaining.min(step);
                std::thread::sleep(sleep_for);
                remaining = remaining.saturating_sub(sleep_for);
            }

            if !task_token.is_cancelled() {
                run(task_token, sender);
            }
        });

        Self {
            cancel_token: Some(cancel_token),
            receiver,
        }
    }
}

impl<T> Drop for ReplaceableTask<T> {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::ReplaceableTask;
    use std::sync::mpsc::TryRecvError;
    use std::time::Duration;

    #[test]
    fn cancel_prevents_pending_task_from_sending() {
        let mut task = ReplaceableTask::spawn(Duration::from_millis(50), |_token, sender| {
            let _ = sender.send("old");
        });

        task.cancel();
        std::thread::sleep(Duration::from_millis(80));

        assert!(matches!(
            task.receiver().try_recv(),
            Err(TryRecvError::Disconnected)
        ));
    }

    #[test]
    fn latest_replacement_can_send() {
        let mut task = ReplaceableTask::spawn(Duration::from_millis(50), |_token, sender| {
            let _ = sender.send("old");
        });
        task.cancel();

        let task = ReplaceableTask::spawn(Duration::from_millis(0), |_token, sender| {
            let _ = sender.send("new");
        });

        assert_eq!(
            task.receiver()
                .recv_timeout(Duration::from_millis(100))
                .unwrap(),
            "new"
        );
    }
}
