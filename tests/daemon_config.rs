use agent_llm_mm::{
    adapters::sqlite::SqliteStore,
    application::daemon::DaemonHandle,
    domain::operation_log::{ActorKind, OperationLogEntry, OperationLogKind, OperationLogStatus},
    ports::OperationLogStore,
    support::config::{AppConfig, DaemonConfig, TransportKind},
};
use chrono::Utc;
use sqlx::sqlite::SqlitePool;
use tempfile::tempdir;
use tokio::time::{Duration, timeout};

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

#[tokio::test]
async fn doctor_reports_observe_only_daemon_diagnostics_without_semantic_writes() {
    let temp_dir = tempdir().expect("temp dir");
    let database_url = sqlite_url(temp_dir.path().join("daemon-observe-only.sqlite"));
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url: database_url.clone(),
        daemon: DaemonConfig {
            enabled: true,
            poll_interval_ms: 250,
            max_concurrent_tasks: 1,
        },
        ..Default::default()
    };

    let _ = agent_llm_mm::run_doctor(config.clone())
        .await
        .expect("initial doctor bootstrap should pass");
    let before = semantic_counts(&database_url).await;

    let report = agent_llm_mm::run_doctor(config)
        .await
        .expect("doctor should report observe-only diagnostics");
    let after = semantic_counts(&database_url).await;

    assert!(report.daemon_enabled);
    assert_eq!(report.daemon_observe_only.mode, "observe_only");
    assert!(report.daemon_observe_only.local_only);
    assert!(!report.daemon_observe_only.write_gate_approved);
    assert!(!report.daemon_observe_only.writes_allowed);
    assert!(!report.daemon_observe_only.remote_listener_enabled);
    assert_eq!(report.daemon_observe_only.in_flight_task_count, 0);
    assert_eq!(report.daemon_observe_only.trigger_candidates_observed, 0);
    assert_eq!(report.daemon_observe_only.read_errors, Vec::<String>::new());
    assert!(
        report
            .daemon_observe_only
            .data_sources
            .contains(&"daemon_config".to_string())
    );
    assert!(
        report
            .daemon_observe_only
            .data_sources
            .contains(&"operation_log".to_string())
    );
    assert_eq!(
        before, after,
        "observe-only daemon diagnostics must not write semantic memory tables"
    );
}

#[tokio::test]
async fn doctor_observe_only_daemon_diagnostics_count_local_operation_candidates() {
    let temp_dir = tempdir().expect("temp dir");
    let database_url = sqlite_url(temp_dir.path().join("daemon-operation-candidates.sqlite"));
    let store = SqliteStore::bootstrap(&database_url).await.unwrap();
    store
        .append_operation(operation_entry(
            "op-failed-tool",
            OperationLogKind::Tool,
            OperationLogStatus::Failed,
        ))
        .await
        .unwrap();
    store
        .append_operation(operation_entry(
            "op-suppressed-trigger",
            OperationLogKind::Trigger,
            OperationLogStatus::Suppressed,
        ))
        .await
        .unwrap();
    for index in 0..30 {
        store
            .append_operation(operation_entry(
                &format!("op-ok-tool-{index}"),
                OperationLogKind::Tool,
                OperationLogStatus::Ok,
            ))
            .await
            .unwrap();
        store
            .append_operation(operation_entry(
                &format!("op-ok-trigger-{index}"),
                OperationLogKind::Trigger,
                OperationLogStatus::Ok,
            ))
            .await
            .unwrap();
    }

    let report = agent_llm_mm::run_doctor(AppConfig {
        database_url,
        daemon: DaemonConfig {
            enabled: true,
            poll_interval_ms: 250,
            max_concurrent_tasks: 1,
        },
        ..Default::default()
    })
    .await
    .expect("doctor should read local operation-log diagnostics");

    assert_eq!(report.daemon_observe_only.trigger_candidates_observed, 1);
    assert_eq!(report.daemon_observe_only.trigger_candidates_suppressed, 1);
    assert_eq!(report.daemon_observe_only.cooldown_status, "observe_only");
    assert_eq!(report.daemon_observe_only.read_errors, Vec::<String>::new());
}

#[tokio::test]
async fn disabled_daemon_handle_exits_without_running_lifecycle_loop() {
    let handle = DaemonHandle::start(DaemonConfig {
        enabled: false,
        poll_interval_ms: 10,
        max_concurrent_tasks: 1,
    });

    assert!(!handle.config_enabled());
    assert_eq!(handle.mode(), "disabled");
    assert_eq!(handle.poll_interval_ms(), 10);
    timeout(Duration::from_millis(250), handle.stop())
        .await
        .expect("disabled daemon should stop promptly");
}

#[tokio::test]
async fn observe_only_daemon_handle_starts_and_stops_without_write_capability() {
    let handle = DaemonHandle::start(DaemonConfig {
        enabled: true,
        poll_interval_ms: 10,
        max_concurrent_tasks: 1,
    });

    assert!(handle.config_enabled());
    assert_eq!(handle.mode(), "observe_only");
    assert!(!handle.writes_allowed());
    assert!(!handle.remote_listener_enabled());
    assert_eq!(handle.poll_interval_ms(), 10);
    timeout(Duration::from_secs(1), handle.stop())
        .await
        .expect("observe-only daemon should stop promptly");
}

#[test]
fn daemon_lifecycle_remains_observe_only_and_has_no_durable_write_or_remote_paths() {
    let source = std::fs::read_to_string("src/application/daemon.rs")
        .expect("daemon source should be readable");

    assert!(source.contains("mode"));
    assert!(source.contains("observe_only"));
    assert!(source.contains("writes_allowed"));
    assert!(source.contains("remote_listener_enabled"));
    assert!(!source.contains("run_reflection"));
    assert!(!source.contains("append_event"));
    assert!(!source.contains("append_claim"));
    assert!(!source.contains("append_reflection"));
    assert!(!source.contains("TcpListener"));
    assert!(!source.contains("start_dashboard_service"));
}

fn sqlite_url(path: impl AsRef<std::path::Path>) -> String {
    format!(
        "sqlite://{}",
        path.as_ref().to_string_lossy().replace('\\', "/")
    )
}

async fn semantic_counts(database_url: &str) -> (i64, i64, i64, i64, i64) {
    let pool = SqlitePool::connect(database_url).await.unwrap();
    let events = table_count(&pool, "events").await;
    let claims = table_count(&pool, "claims").await;
    let reflections = table_count(&pool, "reflections").await;
    let identity_claims = table_count(&pool, "identity_claims").await;
    let commitments = table_count(&pool, "commitments").await;
    (events, claims, reflections, identity_claims, commitments)
}

async fn table_count(pool: &SqlitePool, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn operation_entry(
    operation_id: &str,
    operation_kind: OperationLogKind,
    status: OperationLogStatus,
) -> OperationLogEntry {
    OperationLogEntry {
        operation_id: operation_id.to_string(),
        occurred_at: Utc::now(),
        namespace: Some("self".to_string()),
        actor_kind: ActorKind::System,
        actor_id: "daemon-observe-only-test".to_string(),
        entrypoint: operation_id.to_string(),
        operation_kind,
        status,
        correlation_id: Some(format!("corr-{operation_id}")),
        request_summary_json: None,
        response_summary_json: None,
        diagnostic_summary_json: None,
        redaction_version: 1,
    }
}
