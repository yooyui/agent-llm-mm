use agent_llm_mm::support::config::{AppConfig, DaemonConfig};

#[test]
fn daemon_defaults_to_disabled() {
    let config = AppConfig::default();
    assert!(!config.daemon.enabled);
    assert_eq!(config.daemon.poll_interval_ms, 60_000);
    assert_eq!(config.daemon.max_concurrent_tasks, 1);
}

#[test]
fn daemon_config_rejects_zero_polling_interval() {
    let config = AppConfig {
        daemon: DaemonConfig {
            enabled: true,
            poll_interval_ms: 0,
            max_concurrent_tasks: 1,
        },
        ..Default::default()
    };
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("poll_interval_ms"));
}

#[tokio::test]
async fn doctor_reports_daemon_config_without_starting_daemon() {
    let config = AppConfig::default();
    let report = agent_llm_mm::run_doctor(config).await.unwrap();
    assert!(!report.daemon_enabled);
    assert_eq!(report.daemon_poll_interval_ms, 60_000);
    assert_eq!(report.daemon_max_concurrent_tasks, 1);
}
