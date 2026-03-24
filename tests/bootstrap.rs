use agent_llm_mm::{
    startup_transport_from_default_config,
    support::config::{AppConfig, TransportKind},
};
use std::time::Duration;

#[test]
fn default_config_uses_stdio_transport() {
    let config = AppConfig::default();
    assert_eq!(config.transport, TransportKind::Stdio);
}

#[test]
fn startup_transport_uses_default_config_stdio() {
    assert_eq!(
        startup_transport_from_default_config(),
        TransportKind::Stdio
    );
}

#[tokio::test]
async fn run_uses_stdio_server_path_and_does_not_exit_immediately() {
    let handle = tokio::spawn(agent_llm_mm::run());

    for _ in 0..10 {
        tokio::task::yield_now().await;
        if handle.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    assert!(
        !handle.is_finished(),
        "run() returned immediately instead of serving stdio"
    );

    handle.abort();
    let _ = handle.await;
}
