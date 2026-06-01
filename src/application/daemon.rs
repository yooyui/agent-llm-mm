use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::support::config::DaemonConfig;

pub struct DaemonHandle {
    enabled: bool,
    poll_interval_ms: u64,
    shutdown: Arc<Notify>,
    task: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct DaemonLifecycleProbe {
    running: Arc<AtomicBool>,
}

impl DaemonLifecycleProbe {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

struct RunningGuard {
    running: Arc<AtomicBool>,
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl DaemonHandle {
    pub fn start(config: DaemonConfig) -> Self {
        let enabled = config.enabled;
        let poll_interval_ms = config.poll_interval_ms;
        let shutdown = Arc::new(Notify::new());
        let shutdown_rx = shutdown.clone();
        let running = Arc::new(AtomicBool::new(false));
        let task_running = running.clone();

        let task = tokio::spawn(async move {
            if !config.enabled {
                return;
            }
            task_running.store(true, Ordering::SeqCst);
            let _running_guard = RunningGuard {
                running: task_running,
            };
            let interval = tokio::time::Duration::from_millis(config.poll_interval_ms);
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        // Observe-only idle tick. Candidate reads and any future
                        // write-capable behavior stay behind separate gates.
                    }
                    _ = shutdown_rx.notified() => break,
                }
            }
        });

        Self {
            enabled,
            poll_interval_ms,
            shutdown,
            task: Some(task),
            running,
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

    #[doc(hidden)]
    pub fn lifecycle_probe(&self) -> DaemonLifecycleProbe {
        DaemonLifecycleProbe {
            running: self.running.clone(),
        }
    }

    pub async fn stop(mut self) {
        self.shutdown.notify_one();
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        self.shutdown.notify_one();
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
