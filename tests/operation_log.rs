use agent_llm_mm::{
    adapters::sqlite::SqliteStore,
    domain::operation_log::{
        ActorKind, OperationLogEntry, OperationLogKind, OperationLogStatus, redact_secrets,
    },
    ports::{OperationLogQuery, OperationLogStore},
};
use chrono::{Duration, TimeZone, Utc};

async fn bootstrap_store() -> SqliteStore {
    let path = std::env::temp_dir().join(format!(
        "agent-llm-mm-oplog-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    let database_url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    SqliteStore::bootstrap(&database_url).await.unwrap()
}

fn sample_entry(id: &str, namespace: Option<&str>) -> OperationLogEntry {
    OperationLogEntry {
        operation_id: id.to_string(),
        occurred_at: Utc::now(),
        namespace: namespace.map(str::to_string),
        actor_kind: ActorKind::System,
        actor_id: "mcp-stdio".to_string(),
        entrypoint: "decide_with_snapshot".to_string(),
        operation_kind: OperationLogKind::Tool,
        status: OperationLogStatus::Ok,
        correlation_id: None,
        request_summary_json: None,
        response_summary_json: Some(r#"{"action":"summarize"}"#.to_string()),
        diagnostic_summary_json: None,
        redaction_version: 1,
    }
}

#[tokio::test]
async fn operation_log_persists_and_retrieves_entry() {
    let store = bootstrap_store().await;
    let entry = sample_entry("op-1", Some("self"));

    store.append_operation(entry.clone()).await.unwrap();

    let results = store
        .query_operations(OperationLogQuery {
            limit: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].operation_id, "op-1");
    assert_eq!(results[0].namespace.as_deref(), Some("self"));
    assert_eq!(results[0].actor_kind, ActorKind::System);
    assert_eq!(results[0].operation_kind, OperationLogKind::Tool);
    assert_eq!(results[0].status, OperationLogStatus::Ok);
}

#[tokio::test]
async fn operation_log_queries_by_namespace() {
    let store = bootstrap_store().await;

    store
        .append_operation(sample_entry("op-a", Some("self")))
        .await
        .unwrap();
    store
        .append_operation(sample_entry("op-b", Some("user/default")))
        .await
        .unwrap();
    store
        .append_operation(sample_entry("op-c", Some("self")))
        .await
        .unwrap();

    let results = store
        .query_operations(OperationLogQuery {
            namespace: Some("self".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|e| e.namespace.as_deref() == Some("self"))
    );
}

#[tokio::test]
async fn operation_log_redacts_secrets_from_summary_json() {
    let raw = r#"{"api_key": "sk-secret-12345", "model": "gpt-4o"}"#;
    let redacted = redact_secrets(raw);

    assert!(!redacted.contains("sk-secret-12345"));
    assert!(redacted.contains("[REDACTED]"));
    assert!(redacted.contains("gpt-4o"));
}

#[tokio::test]
async fn operation_log_redacts_summary_json_before_persisting() {
    let store = bootstrap_store().await;
    let mut entry = sample_entry("op-secret", Some("self"));
    entry.request_summary_json =
        Some(r#"{"api_key":"sk-secret-12345","model":"gpt-4o"}"#.to_string());

    store.append_operation(entry).await.unwrap();

    let results = store
        .query_operations(OperationLogQuery {
            limit: Some(1),
            ..Default::default()
        })
        .await
        .unwrap();
    let persisted = results[0]
        .request_summary_json
        .as_deref()
        .expect("request summary should be persisted");
    assert!(!persisted.contains("sk-secret-12345"));
    assert!(persisted.contains("[REDACTED]"));
    assert!(persisted.contains("gpt-4o"));
}

#[tokio::test]
async fn operation_log_queries_by_correlation_id() {
    let store = bootstrap_store().await;

    let mut first = sample_entry("op-first", Some("self"));
    first.occurred_at = Utc.with_ymd_and_hms(2026, 3, 23, 10, 0, 0).unwrap();
    first.correlation_id = Some("cycle-1".to_string());
    let mut second = sample_entry("op-second", Some("self"));
    second.occurred_at = Utc.with_ymd_and_hms(2026, 3, 23, 10, 1, 0).unwrap();
    second.correlation_id = Some("cycle-2".to_string());
    let mut third = sample_entry("op-third", Some("self"));
    third.occurred_at = Utc.with_ymd_and_hms(2026, 3, 23, 10, 2, 0).unwrap();
    third.correlation_id = Some("cycle-1".to_string());

    store.append_operation(first).await.unwrap();
    store.append_operation(second).await.unwrap();
    store.append_operation(third).await.unwrap();

    let results = store
        .query_operations(OperationLogQuery {
            correlation_id: Some("cycle-1".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        results
            .iter()
            .map(|entry| entry.operation_id.as_str())
            .collect::<Vec<_>>(),
        vec!["op-third", "op-first"]
    );
    assert!(
        results
            .iter()
            .all(|entry| entry.correlation_id.as_deref() == Some("cycle-1"))
    );
}

#[tokio::test]
async fn operation_log_queries_by_time_range() {
    let store = bootstrap_store().await;
    let now = Utc::now();

    let mut old_entry = sample_entry("op-old", Some("self"));
    old_entry.occurred_at = now - Duration::hours(2);
    let mut new_entry = sample_entry("op-new", Some("self"));
    new_entry.occurred_at = now;

    store.append_operation(old_entry).await.unwrap();
    store.append_operation(new_entry).await.unwrap();

    let results = store
        .query_operations(OperationLogQuery {
            after: Some(now - Duration::hours(1)),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].operation_id, "op-new");
}
