use std::sync::Arc;

use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::support::config::DaemonConfig;

pub struct DaemonHandle {
    enabled: bool,
    poll_interval_ms: u64,
    shutdown: Arc<Notify>,
    task: JoinHandle<()>,
}

impl DaemonHandle {
    pub fn start(config: DaemonConfig) -> Self {
        let enabled = config.enabled;
        let poll_interval_ms = config.poll_interval_ms;
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

        Self {
            enabled,
            poll_interval_ms,
            shutdown,
            task,
        }
    }

    pub fn config_enabled(&self) -> bool {
        self.enabled
    }

    pub fn mode(&self) -> &'static str {
        if self.enabled {
            "observe_only"
        } else {
            "disabled"
        }
    }

    pub fn poll_interval_ms(&self) -> u64 {
        self.poll_interval_ms
    }

    pub fn writes_allowed(&self) -> bool {
        false
    }

    pub fn remote_listener_enabled(&self) -> bool {
        false
    }

    pub async fn stop(self) {
        self.shutdown.notify_one();
        let _ = self.task.await;
    }
}
