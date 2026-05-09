use std::sync::Arc;

use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::support::config::DaemonConfig;

pub struct DaemonHandle {
    shutdown: Arc<Notify>,
    task: JoinHandle<()>,
}

impl DaemonHandle {
    pub fn start(config: DaemonConfig) -> Self {
        let shutdown = Arc::new(Notify::new());
        let shutdown_rx = shutdown.clone();

        let task = tokio::spawn(async move {
            if !config.enabled {
                return;
            }
            let interval = tokio::time::Duration::from_millis(config.poll_interval_ms);
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        // Future: poll operation log and trigger reflection
                    }
                    _ = shutdown_rx.notified() => break,
                }
            }
        });

        Self { shutdown, task }
    }

    pub async fn stop(self) {
        self.shutdown.notify_one();
        let _ = self.task.await;
    }
}
