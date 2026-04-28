use std::{
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
};

use agent_llm_mm::support::config::{CONFIG_PATH_ENV_VAR, DATABASE_URL_ENV_VAR};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Row, sqlite::SqlitePool};
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};

#[tokio::test]
async fn server_exposes_expected_tools_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let tools = client.list_all_tools().await.unwrap();
    let mut names = tools
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect::<Vec<_>>();
    names.sort();

    assert_eq!(
        names,
        vec![
            "build_self_snapshot".to_string(),
            "decide_with_snapshot".to_string(),
            "ingest_interaction".to_string(),
            "run_reflection".to_string(),
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ingest_interaction_returns_success_even_when_best_effort_auto_reflection_fails() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "not valid self revision json"
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    for (episode_reference, summary, trigger_hints) in [
        (
            "episode:auto-reflect-nonfatal-0",
            "first rollback after violating a hard commitment",
            json!([]),
        ),
        (
            "episode:auto-reflect-nonfatal-1",
            "rollback after violating a hard commitment",
            json!(["failure", "rollback"]),
        ),
    ] {
        let response = client
            .call_tool(
                "ingest_interaction",
                json!({
                    "event": {
                        "owner": "Self_",
                        "kind": "Action",
                        "summary": summary
                    },
                    "claim_drafts": [],
                    "episode_reference": episode_reference,
                    "trigger_hints": trigger_hints
                }),
            )
            .await
            .unwrap();

        let event_id = response
            .get("result")
            .and_then(|value| value.get("structuredContent"))
            .and_then(|value| value.get("event_id"))
            .and_then(Value::as_str);
        assert!(
            event_id.is_some(),
            "ingest should still succeed when best-effort auto-reflection fails: {response:?}"
        );
        assert!(
            response.get("error").is_none(),
            "post-ingest auto-reflection failure must not surface as MCP error: {response:?}"
        );
    }

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let event_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(event_count, 2);
    assert_eq!(reflection_count, 0);
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ingest_interaction_auto_reflects_once_and_does_not_recurse_inside_run_reflection() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"Repeated rollback should tighten commitments.","machine_patch":{"commitment_patch":{"commitments":["prefer:reflect_before_repeating_rollback"]}}}"#
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "first rollback after violating a hard commitment"
                },
                "claim_drafts": [],
                "episode_reference": "episode:auto-reflect-0"
            }),
        )
        .await
        .unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "rollback after violating a hard commitment"
                },
                "claim_drafts": [],
                "episode_reference": "episode:auto-reflect-1",
                "trigger_hints": ["failure", "rollback"]
            }),
        )
        .await
        .unwrap();

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let trigger_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT trigger_type, namespace FROM reflection_trigger_ledger ORDER BY rowid ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(reflection_count, 1);
    assert_eq!(
        trigger_rows,
        vec![("failure".to_string(), "self".to_string())]
    );
    assert_eq!(stub.request_count().await, 1);

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "A direct reflection target exists for recursion checking."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:auto-reflect-direct-reflection-target"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Explicit MCP reflection should not recurse into auto-reflection."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": null
            }),
        )
        .await
        .unwrap();
    assert!(
        reflection.get("error").is_none(),
        "explicit run_reflection should still succeed: {reflection:?}"
    );

    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 2);
    assert_eq!(trigger_ledger_count, 1);
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ingest_interaction_can_trigger_conflict_auto_reflection_when_explicit_conflict_hints_present()
 {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"Conflict evidence suggests tighter commitment hygiene.","machine_patch":{"commitment_patch":{"commitments":["prefer:confirm_conflicting_commitment_updates_before_overwrite"]}}}"#
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let response = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "self attempted a commitment overwrite that may need review"
                },
                "claim_drafts": [],
                "episode_reference": "episode:ingest-conflict-auto-reflect",
                "trigger_hints": ["conflict", "commitment"]
            }),
        )
        .await
        .unwrap();

    let event_id = response
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str);
    assert!(
        response.get("error").is_none(),
        "ingest must still succeed when conflict auto-reflection runs: {response:?}"
    );
    assert!(
        event_id.is_some(),
        "ingest should still succeed when conflict auto-reflection runs: {response:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT trigger_type, status FROM reflection_trigger_ledger ORDER BY rowid ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(reflection_count, 1);
    assert_eq!(
        trigger_rows,
        vec![("conflict".to_string(), "handled".to_string())]
    );
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ingest_interaction_returns_success_even_when_conflict_auto_reflection_fails() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "not valid self revision json"
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let response = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "self attempted a commitment overwrite that may need review"
                },
                "claim_drafts": [],
                "episode_reference": "episode:ingest-conflict-auto-reflect-nonfatal",
                "trigger_hints": ["conflict", "commitment"]
            }),
        )
        .await
        .unwrap();

    assert!(
        response.get("error").is_none(),
        "ingest must still succeed when best-effort conflict auto-reflection fails: {response:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let event_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(event_count, 1);
    assert_eq!(reflection_count, 0);
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ingest_interaction_does_not_auto_reflect_conflict_without_explicit_conflict_hints() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"Conflict evidence suggests tighter commitment hygiene.","machine_patch":{"commitment_patch":{"commitments":["prefer:confirm_conflicting_commitment_updates_before_overwrite"]}}}"#
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let response = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "self attempted a conflicting commitment overwrite"
                },
                "claim_drafts": [],
                "episode_reference": "episode:ingest-conflict-without-hints"
            }),
        )
        .await
        .unwrap();

    assert!(
        response.get("error").is_none(),
        "ingest must still succeed without explicit conflict hints: {response:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 0);
    assert_eq!(trigger_ledger_count, 0);
    assert_eq!(stub.request_count().await, 0);
}

#[tokio::test]
async fn ingest_interaction_rejects_ambiguous_auto_reflect_scope_before_writing() {
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_database()
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let response = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "Self_",
                    "kind": "Action",
                    "summary": "Mixed claim scopes should not depend on draft ordering."
                },
                "claim_drafts": [
                    {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "subject": "project.memory",
                        "predicate": "needs",
                        "object": "structure",
                        "mode": "Observed"
                    },
                    {
                        "owner": "World",
                        "namespace": "world",
                        "subject": "weather",
                        "predicate": "is",
                        "object": "rainy",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:auto-reflect-ambiguous-scope",
                "trigger_hints": ["failure", "rollback"]
            }),
        )
        .await
        .unwrap();

    let error = response
        .get("error")
        .expect("ambiguous mixed-scope ingest should be rejected before any write");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let event_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(event_count, 0);
}

#[tokio::test]
async fn stdio_tools_share_runtime_state_across_calls() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user asked for stronger memory."
                },
                "claim_drafts": [
                    {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "subject": "project.memory",
                        "predicate": "needs",
                        "object": "structure",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:task-7"
            }),
        )
        .await
        .unwrap();
    let event_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .unwrap();
    assert!(!event_id.is_empty());

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let claims = snapshot
        .get("claims")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        claims.contains(&"project/agent-llm-mm:project.memory needs structure"),
        "snapshot claims missing ingested claim: {claims:?}"
    );

    let evidence = snapshot
        .get("evidence")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(evidence.len(), 1, "expected one evidence reference");
    assert!(
        evidence[0].starts_with("event:"),
        "unexpected evidence reference: {:?}",
        evidence
    );

    let episodes = snapshot
        .get("episodes")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        episodes.contains(&"episode:task-7"),
        "snapshot episodes missing ingested episode: {episodes:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_self_snapshot_can_trigger_periodic_auto_reflection_once_for_explicit_namespace() {
    let stub_response = r#"{"should_reflect":true,"rationale":"Periodic review should tighten commitments after repeated project evidence.","machine_patch":{"identity_patch":null,"commitment_patch":{"commitments":["prefer:review_project_commitments_before_repeating_snapshot_builds"]}}}"#;
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": stub_response
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    for (summary, episode_reference) in [
        (
            "Observed one project-memory maintenance gap for periodic review.",
            "episode:build-self-snapshot-periodic-0",
        ),
        (
            "Observed another project-memory maintenance gap for periodic review.",
            "episode:build-self-snapshot-periodic-1",
        ),
    ] {
        let ingest = client
            .call_tool(
                "ingest_interaction",
                json!({
                    "event": {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "kind": "Observation",
                        "summary": summary
                    },
                    "claim_drafts": [
                        {
                            "owner": "World",
                            "namespace": "project/agent-llm-mm",
                            "subject": "project.memory",
                            "predicate": "needs",
                            "object": "periodic-review",
                            "mode": "Observed"
                        }
                    ],
                    "episode_reference": episode_reference
                }),
            )
            .await
            .unwrap();
        assert!(
            ingest.get("error").is_none(),
            "seed ingest should succeed without depending on auto-reflection: {ingest:?}"
        );
    }

    let first = client
        .call_tool(
            "build_self_snapshot",
            json!({
                "budget": 4,
                "auto_reflect_namespace": "project/agent-llm-mm"
            }),
        )
        .await
        .unwrap();
    let second = client
        .call_tool(
            "build_self_snapshot",
            json!({
                "budget": 4,
                "auto_reflect_namespace": "project/agent-llm-mm"
            }),
        )
        .await
        .unwrap();

    for response in [&first, &second] {
        assert!(
            response.get("error").is_none(),
            "build_self_snapshot should still return a snapshot object: {response:?}"
        );
        let snapshot = response
            .get("result")
            .and_then(|value| value.get("structuredContent"))
            .and_then(|value| value.get("snapshot"));
        assert!(
            snapshot.is_some_and(Value::is_object),
            "build_self_snapshot must preserve the snapshot payload shape: {response:?}"
        );
    }

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_rows = sqlx::query(
        r#"
        SELECT trigger_type, namespace, trigger_key, status
        FROM reflection_trigger_ledger
        ORDER BY rowid ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| {
        (
            row.get::<String, _>("trigger_type"),
            row.get::<String, _>("namespace"),
            row.get::<String, _>("trigger_key"),
            row.get::<String, _>("status"),
        )
    })
    .collect::<Vec<_>>();

    assert_eq!(reflection_count, 1);
    assert_eq!(
        trigger_rows,
        vec![
            (
                "periodic".to_string(),
                "project/agent-llm-mm".to_string(),
                "project/agent-llm-mm:periodic".to_string(),
                "handled".to_string(),
            ),
            (
                "periodic".to_string(),
                "project/agent-llm-mm".to_string(),
                "project/agent-llm-mm:periodic".to_string(),
                "suppressed".to_string(),
            ),
        ],
        "explicit build_self_snapshot wiring should record one handled periodic trigger and one suppressed retry"
    );
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_self_snapshot_returns_snapshot_when_best_effort_periodic_auto_reflection_fails() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "not valid self revision json"
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    for (summary, episode_reference) in [
        (
            "Observed one project-memory maintenance gap before periodic snapshot reflection.",
            "episode:build-self-snapshot-periodic-nonfatal-0",
        ),
        (
            "Observed another project-memory maintenance gap before periodic snapshot reflection.",
            "episode:build-self-snapshot-periodic-nonfatal-1",
        ),
    ] {
        let ingest = client
            .call_tool(
                "ingest_interaction",
                json!({
                    "event": {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "kind": "Observation",
                        "summary": summary
                    },
                    "claim_drafts": [
                        {
                            "owner": "World",
                            "namespace": "project/agent-llm-mm",
                            "subject": "project.memory",
                            "predicate": "needs",
                            "object": "periodic-review",
                            "mode": "Observed"
                        }
                    ],
                    "episode_reference": episode_reference
                }),
            )
            .await
            .unwrap();
        assert!(
            ingest.get("error").is_none(),
            "seed ingest should succeed before best-effort periodic auto-reflection is attempted: {ingest:?}"
        );
    }

    let response = client
        .call_tool(
            "build_self_snapshot",
            json!({
                "budget": 4,
                "auto_reflect_namespace": "project/agent-llm-mm"
            }),
        )
        .await
        .unwrap();

    assert!(
        response.get("error").is_none(),
        "build_self_snapshot should not surface periodic auto-reflection failures as MCP errors: {response:?}"
    );
    let snapshot = response
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"));
    assert!(
        snapshot.is_some_and(Value::is_object),
        "build_self_snapshot should still return a snapshot object after best-effort periodic auto-reflection fails: {response:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 0);
    assert_eq!(trigger_ledger_count, 0);
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_self_snapshot_does_not_auto_reflect_without_explicit_namespace() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": r#"{"should_reflect":true,"rationale":"Periodic review is warranted.","machine_patch":{"commitment_patch":{"commitments":["prefer:periodic_project_review"]}}}"#
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "Seed an episode before a snapshot without auto-reflect namespace."
                },
                "claim_drafts": [
                    {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "subject": "project.memory",
                        "predicate": "needs",
                        "object": "structure",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:build-self-snapshot-without-namespace"
            }),
        )
        .await
        .unwrap();
    assert!(
        ingest.get("error").is_none(),
        "seed ingest should succeed before a namespace-free snapshot: {ingest:?}"
    );

    let response = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();

    assert!(
        response.get("error").is_none(),
        "build_self_snapshot must still succeed without explicit auto_reflect_namespace: {response:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 0);
    assert_eq!(trigger_ledger_count, 0);
    assert_eq!(stub.request_count().await, 0);
}

#[tokio::test]
async fn conflicting_reflection_over_stdio_removes_claim_from_active_snapshot() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user described a role conflict."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:task-8"
            }),
        )
        .await
        .unwrap();
    let event_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .unwrap()
        .to_string();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "This reflection conflicts with the previous claim."
                },
                "supersede_claim_id": format!("{event_id}:claim:0"),
                "replacement_claim": null
            }),
        )
        .await
        .unwrap();
    let replacement_claim_id = reflection
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("replacement_claim_id"));
    assert!(
        replacement_claim_id.is_some_and(Value::is_null),
        "conflicting reflection should not create a replacement claim: {reflection:?}"
    );

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let claims = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .and_then(|value| value.get("claims"))
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert!(
        !claims.contains(&"self:self.role is architect"),
        "conflicting reflection should remove disputed claims from active snapshot: {claims:?}"
    );
}

#[tokio::test]
async fn fresh_stdio_runtime_blocks_forbidden_action_with_seeded_commitment() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Observation",
                    "summary": "Bootstrap one evidence event so the snapshot can be built."
                },
                "claim_drafts": [],
                "episode_reference": "episode:task-8-gate"
            }),
        )
        .await
        .unwrap();

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let commitments = snapshot
        .get("commitments")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        commitments.contains(&"forbid:write_identity_core_directly"),
        "fresh stdio runtime should seed the baseline commitment: {commitments:?}"
    );

    let decision = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "attempt a forbidden direct identity write",
                "action": "write_identity_core_directly",
                "snapshot": snapshot,
            }),
        )
        .await
        .unwrap();

    let blocked = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("blocked"))
        .and_then(Value::as_bool)
        .unwrap();
    let model_decision = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("decision"));

    assert!(blocked, "baseline commitment should block forbidden action");
    assert!(
        model_decision.is_some_and(Value::is_null),
        "blocked decisions should not call the model: {decision:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocked_decide_with_snapshot_does_not_auto_reflect_conflict_hints() {
    let stub_response = r#"{"should_reflect":true,"rationale":"Conflict suggests tighter commitment hygiene.","machine_patch":{"identity_patch":null,"commitment_patch":{"commitments":["prefer:confirm_conflicting_commitment_updates_before_overwrite"]}}}"#;
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": stub_response
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Observation",
                    "summary": "Bootstrap one evidence event so the blocked decision can still build a snapshot."
                },
                "claim_drafts": [],
                "episode_reference": "episode:blocked-decide-with-snapshot-conflict-auto-reflect"
            }),
        )
        .await
        .unwrap();

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let decision = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "attempt a forbidden direct identity write with conflict hints",
                "action": "write_identity_core_directly",
                "snapshot": snapshot,
                "auto_reflect_namespace": "self",
                "trigger_hints": ["conflict", "commitment"]
            }),
        )
        .await
        .unwrap();

    let blocked = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("blocked"))
        .and_then(Value::as_bool)
        .unwrap();
    let model_decision = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("decision"));

    assert!(
        decision.get("error").is_none(),
        "blocked decide_with_snapshot must not surface MCP errors: {decision:?}"
    );
    assert!(
        blocked,
        "commitment gate should still block this action: {decision:?}"
    );
    assert!(
        model_decision.is_some_and(Value::is_null),
        "blocked decisions must preserve the original null decision payload: {decision:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 0);
    assert_eq!(trigger_ledger_count, 0);
    assert_eq!(stub.request_count().await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unhinted_decide_with_snapshot_does_not_auto_reflect_conflict_from_existing_evidence() {
    let stub_response = r#"{"should_reflect":true,"rationale":"Conflict suggests tighter commitment hygiene.","machine_patch":{"identity_patch":null,"commitment_patch":{"commitments":["prefer:confirm_conflicting_commitment_updates_before_overwrite"]}}}"#;
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": stub_response
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "Seed one evidence event before an unhinted decision."
                },
                "claim_drafts": [],
                "episode_reference": "episode:decide-with-snapshot-unhinted-conflict-auto-reflect"
            }),
        )
        .await
        .unwrap();

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let decision = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "resolve a routine commitment update without explicit conflict hints",
                "action": "overwrite_commitment",
                "snapshot": snapshot,
                "auto_reflect_namespace": "self"
            }),
        )
        .await
        .unwrap();

    let blocked = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("blocked"))
        .and_then(Value::as_bool)
        .unwrap();
    let model_decision = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("decision"))
        .cloned();

    assert!(
        decision.get("error").is_none(),
        "unhinted decide_with_snapshot must not surface MCP errors: {decision:?}"
    );
    assert!(
        !blocked,
        "unhinted decide_with_snapshot should preserve the non-blocked decision flow: {decision:?}"
    );
    assert_eq!(
        model_decision,
        Some(json!({ "action": stub_response })),
        "decision payload must remain the original decision response when conflict auto-reflection is not hinted: {decision:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 0);
    assert_eq!(trigger_ledger_count, 0);
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn decide_with_snapshot_does_not_auto_reflect_conflict_without_explicit_namespace() {
    let stub_response = r#"{"should_reflect":true,"rationale":"Conflict suggests tighter commitment hygiene.","machine_patch":{"identity_patch":null,"commitment_patch":{"commitments":["prefer:confirm_conflicting_commitment_updates_before_overwrite"]}}}"#;
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": stub_response
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "Seed one evidence event before a namespaceless conflict-hinted decision."
                },
                "claim_drafts": [],
                "episode_reference": "episode:decide-with-snapshot-without-namespace"
            }),
        )
        .await
        .unwrap();

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let decision = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "resolve a commitment overwrite with explicit conflict hints but no namespace opt-in",
                "action": "overwrite_commitment",
                "snapshot": snapshot,
                "trigger_hints": ["conflict", "commitment"]
            }),
        )
        .await
        .unwrap();

    assert!(
        decision.get("error").is_none(),
        "decide_with_snapshot must still succeed without explicit auto_reflect_namespace: {decision:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(reflection_count, 0);
    assert_eq!(trigger_ledger_count, 0);
    assert_eq!(stub.request_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn decide_with_snapshot_can_trigger_conflict_auto_reflection_without_breaking_decision_flow()
{
    let stub_response = r#"{"should_reflect":true,"rationale":"Conflict suggests tighter commitment hygiene.","machine_patch":{"identity_patch":null,"commitment_patch":{"commitments":["prefer:confirm_conflicting_commitment_updates_before_overwrite"]}}}"#;
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": stub_response
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_config_and_database(config)
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "Seed one evidence event before resolving a conflicting commitment update."
                },
                "claim_drafts": [],
                "episode_reference": "episode:decide-with-snapshot-conflict-auto-reflect"
            }),
        )
        .await
        .unwrap();

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let decision = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "resolve a conflicting commitment update",
                "action": "overwrite_commitment",
                "snapshot": snapshot,
                "auto_reflect_namespace": "self",
                "trigger_hints": ["conflict", "commitment"]
            }),
        )
        .await
        .unwrap();

    let blocked = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("blocked"))
        .and_then(Value::as_bool)
        .unwrap();
    let model_decision = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("decision"))
        .cloned();

    assert!(
        decision.get("error").is_none(),
        "conflict auto-reflection must not surface as an MCP error: {decision:?}"
    );
    assert!(
        !blocked,
        "conflict auto-reflection should not block a successful decision flow: {decision:?}"
    );
    assert_eq!(
        model_decision,
        Some(json!({ "action": stub_response })),
        "decision payload must remain in the original shape after conflict auto-reflection: {decision:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let reflection_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflections")
        .fetch_one(&pool)
        .await
        .unwrap();
    let trigger_ledger_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reflection_trigger_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();
    let trigger_namespaces = sqlx::query_scalar::<_, String>(
        "SELECT namespace FROM reflection_trigger_ledger ORDER BY rowid ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(reflection_count, 1);
    assert_eq!(trigger_ledger_count, 1);
    assert_eq!(trigger_namespaces, vec!["self".to_string()]);
    assert_eq!(stub.request_count().await, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn decide_with_snapshot_over_stdio_uses_openai_compatible_provider_from_config_file() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "provider_selected_action"
                }
            }]
        }),
    )
    .await;
    let config = format!(
        r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "{}"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 30000
"#,
        stub.base_url()
    );
    let mut client = test_support::spawn_stdio_client_with_config(config)
        .await
        .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let response = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "summarize current memory",
                "action": "read_identity_core",
                "snapshot": {
                    "identity": ["identity:self=architect"],
                    "commitments": [],
                    "claims": ["self.role is architect"],
                    "evidence": ["event:evt-1"],
                    "episodes": ["episode:task-6"]
                }
            }),
        )
        .await
        .unwrap();

    let action = response
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("decision"))
        .and_then(|value| value.get("action"))
        .and_then(Value::as_str);

    assert_eq!(
        action,
        Some("provider_selected_action"),
        "unexpected stdio response: {response:?}"
    );
    assert_eq!(
        stub.last_request_path().await.as_deref(),
        Some("/chat/completions")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    let config = r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[dashboard]
enabled = true
host = "127.0.0.1"
port = __DASHBOARD_PORT__
event_capacity = 50
required = true
"#
    .replace("__DASHBOARD_PORT__", &port.to_string());
    let mut client = test_support::spawn_stdio_client_with_config(config)
        .await
        .expect("client");

    let tools = client.list_all_tools().await.expect("list tools");
    assert_eq!(tools.len(), 4);

    let health: serde_json::Value = reqwest::get(format!("http://127.0.0.1:{port}/api/health"))
        .await
        .expect("dashboard health response")
        .json()
        .await
        .expect("dashboard health json");
    assert_eq!(health["status"], "ok");

    let response = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "Dashboard should record this MCP operation."
                },
                "claim_drafts": [],
                "episode_reference": "episode:dashboard-stdio-smoke"
            }),
        )
        .await
        .expect("ingest response");

    assert!(
        response.get("result").is_some(),
        "dashboard logs must not corrupt MCP stdout: {response:?}"
    );

    let events: serde_json::Value =
        reqwest::get(format!("http://127.0.0.1:{port}/api/events?limit=10"))
            .await
            .expect("dashboard events response")
            .json()
            .await
            .expect("dashboard events json");
    let event_operations = events
        .as_array()
        .expect("events array")
        .iter()
        .filter_map(|event| event.get("operation").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        event_operations.contains(&"ingest_interaction"),
        "dashboard should record the MCP tool operation: {events:?}"
    );
}

#[tokio::test]
async fn invalid_namespace_is_reported_as_invalid_params_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let response = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "This should fail because the namespace is incompatible."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "namespace": "user/default",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": null
            }),
        )
        .await
        .unwrap();

    let error = response
        .get("error")
        .expect("invalid params should return error");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));
}

#[tokio::test]
async fn inferred_replacement_reflection_with_evidence_is_accepted_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let mut evidence_event_ids = Vec::new();
    for summary in [
        "The first external observation supports the inferred replacement.",
        "The second external observation independently supports the inferred replacement.",
    ] {
        let response = client
            .call_tool(
                "ingest_interaction",
                json!({
                    "event": {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "kind": "Observation",
                        "summary": summary
                    },
                    "claim_drafts": [],
                    "episode_reference": "episode:reflection-evidence-source"
                }),
            )
            .await
            .unwrap();
        let event_id = response
            .get("result")
            .and_then(|value| value.get("structuredContent"))
            .and_then(|value| value.get("event_id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap();
        evidence_event_ids.push(event_id);
    }

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user suggested the role may have evolved."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-evidence"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Two external observations support promoting the inferred replacement."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": {
                    "owner": "Self_",
                    "subject": "self.role",
                    "predicate": "is",
                    "object": "principal_architect",
                    "mode": "Inferred"
                },
                "replacement_evidence_event_ids": evidence_event_ids
            }),
        )
        .await
        .unwrap();

    let replacement_claim_id = reflection
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("replacement_claim_id"))
        .and_then(Value::as_str);
    assert!(
        replacement_claim_id.is_some_and(|claim_id| claim_id.ends_with(":replacement")),
        "replacement claim id should be present and use the reflection replacement suffix: {reflection:?}"
    );
}

#[tokio::test]
async fn missing_replacement_evidence_event_ids_are_invalid_params_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user suggested the role may have evolved."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-missing-evidence"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Unknown evidence ids should be rejected before persistence."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": {
                    "owner": "Self_",
                    "subject": "self.role",
                    "predicate": "is",
                    "object": "principal_architect",
                    "mode": "Inferred"
                },
                "replacement_evidence_event_ids": ["evt-missing"]
            }),
        )
        .await
        .unwrap();

    let error = reflection
        .get("error")
        .expect("unknown evidence ids should be reported as invalid params");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));
}

#[tokio::test]
async fn reflected_claim_replacement_query_is_accepted_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    for summary in ["World observed update A.", "World observed update B."] {
        client
            .call_tool(
                "ingest_interaction",
                json!({
                    "event": {
                        "owner": "World",
                        "namespace": "project/agent-llm-mm",
                        "kind": "Observation",
                        "summary": summary
                    },
                    "claim_drafts": [],
                    "episode_reference": "episode:reflection-query-source"
                }),
            )
            .await
            .unwrap();
    }

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The role may still evolve."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-query-target"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Query-based evidence lookup should still allow replacement."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": {
                    "owner": "Self_",
                    "subject": "self.role",
                    "predicate": "is",
                    "object": "principal_architect",
                    "mode": "Inferred"
                },
                "replacement_evidence_query": {
                    "namespace": "project/agent-llm-mm",
                    "owner": "World",
                    "kind": "Observation",
                    "limit": 2
                }
            }),
        )
        .await
        .unwrap();

    let replacement_claim_id = reflection
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("replacement_claim_id"))
        .and_then(Value::as_str);

    assert!(
        replacement_claim_id.is_some_and(|claim_id| claim_id.ends_with(":replacement")),
        "query-based replacement should resolve a replacement claim id: {reflection:?}"
    );
}

#[tokio::test]
async fn reflected_claim_replacement_query_without_matches_is_invalid_params_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "No world observations are stored for this test."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-query-missing"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Query returns nothing, so this should be rejected."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": {
                    "owner": "Self_",
                    "subject": "self.role",
                    "predicate": "is",
                    "object": "principal_architect",
                    "mode": "Inferred"
                },
                "replacement_evidence_query": {
                    "namespace": "project/missing",
                    "owner": "World",
                    "kind": "Conversation",
                    "limit": 3
                }
            }),
        )
        .await
        .unwrap();

    let error = reflection
        .get("error")
        .expect("query without matching evidence should return invalid params");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));
}

#[tokio::test]
async fn reflection_identity_and_commitment_updates_are_applied_and_audited_over_stdio() {
    let (mut client, database_url, _database_dir) =
        test_support::spawn_stdio_client_with_database()
            .await
            .unwrap();
    let _ = client.list_all_tools().await.unwrap();

    for summary in [
        "World observed stronger evidence for the updated role.",
        "World observed the preference for evidence-backed identity changes.",
    ] {
        client
            .call_tool(
                "ingest_interaction",
                json!({
                    "event": {
                        "owner": "World",
                        "kind": "Observation",
                        "summary": summary
                    },
                    "claim_drafts": [],
                    "episode_reference": "episode:reflection-deeper-updates-source"
                }),
            )
            .await
            .unwrap();
    }

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user clarified that the role has shifted."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-deeper-updates-target"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Shared evidence should update the replacement claim, identity, and commitments."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": {
                    "owner": "Self_",
                    "subject": "self.role",
                    "predicate": "is",
                    "object": "staff_architect",
                    "mode": "Observed"
                },
                "replacement_evidence_query": {
                    "owner": "World",
                    "kind": "Observation",
                    "limit": 2
                },
                "identity_update": {
                    "canonical_claims": [
                        "identity:self=staff_architect",
                        "identity:style=evidence_first"
                    ]
                },
                "commitment_updates": [
                    {
                        "owner": "Self_",
                        "description": "prefer:evidence_backed_identity_updates"
                    },
                    {
                        "owner": "Self_",
                        "description": "forbid:write_identity_core_directly"
                    }
                ]
            }),
        )
        .await
        .unwrap();

    let reflection_id = reflection
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("reflection_id"))
        .and_then(Value::as_str)
        .unwrap()
        .to_string();
    let replacement_claim_id = reflection
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("replacement_claim_id"))
        .and_then(Value::as_str);
    assert!(
        replacement_claim_id.is_some_and(|claim_id| claim_id.ends_with(":replacement")),
        "deeper reflection should still create a replacement claim: {reflection:?}"
    );

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 8 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let identity = snapshot
        .get("identity")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let commitments = snapshot
        .get("commitments")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let claims = snapshot
        .get("claims")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(
        identity,
        vec![
            "identity:self=staff_architect",
            "identity:style=evidence_first",
        ]
    );
    assert_eq!(
        commitments,
        vec![
            "prefer:evidence_backed_identity_updates",
            "forbid:write_identity_core_directly",
        ]
    );
    assert!(
        claims.contains(&"self:self.role is staff_architect"),
        "replacement claim should be visible in the snapshot: {claims:?}"
    );

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let audit_row = sqlx::query(
        r#"
        SELECT
            supporting_evidence_event_ids,
            requested_identity_update,
            requested_commitment_updates
        FROM reflections
        WHERE reflection_id = ?
        "#,
    )
    .bind(&reflection_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let supporting_evidence_event_ids = serde_json::from_str::<Vec<String>>(
        &audit_row.get::<String, _>("supporting_evidence_event_ids"),
    )
    .unwrap();
    let requested_identity_update = serde_json::from_str::<serde_json::Value>(
        &audit_row.get::<String, _>("requested_identity_update"),
    )
    .unwrap();
    let requested_commitment_updates = serde_json::from_str::<Vec<serde_json::Value>>(
        &audit_row.get::<String, _>("requested_commitment_updates"),
    )
    .unwrap();

    assert_eq!(supporting_evidence_event_ids.len(), 2);
    assert_eq!(
        requested_identity_update,
        json!({
            "canonical_claims": [
                "identity:self=staff_architect",
                "identity:style=evidence_first"
            ]
        })
    );
    assert_eq!(
        requested_commitment_updates,
        vec![
            json!({
                "owner": "Self_",
                "description": "prefer:evidence_backed_identity_updates"
            }),
            json!({
                "owner": "Self_",
                "description": "forbid:write_identity_core_directly"
            }),
        ]
    );
}

#[tokio::test]
async fn reflection_identity_or_commitment_updates_require_evidence_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "A claim exists, but no supporting evidence event is provided."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-deeper-updates-missing-evidence"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Identity-only updates still need resolved evidence."
                },
                "supersede_claim_id": superseded_claim_id.clone(),
                "replacement_claim": null,
                "identity_update": {
                    "canonical_claims": ["identity:self=principal_architect"]
                }
            }),
        )
        .await
        .unwrap();

    let error = reflection
        .get("error")
        .expect("identity update without evidence should return invalid params");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Commitment-only updates still need resolved evidence."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": null,
                "commitment_updates": [
                    {
                        "owner": "Self_",
                        "description": "prefer:reflect_before_identity_changes"
                    }
                ]
            }),
        )
        .await
        .unwrap();

    let error = reflection
        .get("error")
        .expect("missing evidence for deeper reflection updates should return invalid params");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));
}

#[tokio::test]
async fn replacement_evidence_query_limit_overflow_is_invalid_params_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "World",
                    "kind": "Observation",
                    "summary": "A matching observation exists, so overflow must not be masked as an empty-query error."
                },
                "claim_drafts": [],
                "episode_reference": "episode:reflection-limit-overflow-source"
            }),
        )
        .await
        .unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "Overflowing query limits should be rejected."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:reflection-limit-overflow"
            }),
        )
        .await
        .unwrap();
    let superseded_claim_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .map(|event_id| format!("{event_id}:claim:0"))
        .unwrap();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "Oversized evidence query limits should fail before SQLite treats them as unbounded."
                },
                "supersede_claim_id": superseded_claim_id,
                "replacement_claim": {
                    "owner": "Self_",
                    "subject": "self.role",
                    "predicate": "is",
                    "object": "principal_architect",
                    "mode": "Observed"
                },
                "replacement_evidence_query": {
                    "owner": "World",
                    "kind": "Observation",
                    "limit": 9223372036854775808u64
                }
            }),
        )
        .await
        .unwrap();

    let error = reflection
        .get("error")
        .expect("overflowing query limit should be reported as invalid params");
    assert_eq!(error.get("code").and_then(Value::as_i64), Some(-32602));
}

mod test_support {
    use super::*;

    pub async fn spawn_stdio_client() -> io::Result<StdioClient> {
        let database = database_override()?;
        StdioClient::spawn(&database.url, Some(database.temp_dir))
    }

    pub async fn spawn_stdio_client_with_config(
        config_template: String,
    ) -> io::Result<StdioClient> {
        let database = database_override()?;
        let config_path = database.temp_dir.path().join("agent-llm-mm.local.toml");
        let config = config_template.replace("__DATABASE_URL__", &database.url);
        std::fs::write(&config_path, config)?;

        StdioClient::spawn_with_env(
            Some(database.temp_dir),
            &[(
                CONFIG_PATH_ENV_VAR,
                config_path.to_string_lossy().into_owned(),
            )],
        )
    }

    pub async fn spawn_stdio_client_with_config_and_database(
        config_template: String,
    ) -> io::Result<(StdioClient, String, TempDir)> {
        let database = database_override()?;
        let url = database.url.clone();
        let config_path = database.temp_dir.path().join("agent-llm-mm.local.toml");
        let config = config_template.replace("__DATABASE_URL__", &url);
        std::fs::write(&config_path, config)?;
        let temp_dir = database.temp_dir;
        let client = StdioClient::spawn_with_env(
            None,
            &[(
                CONFIG_PATH_ENV_VAR,
                config_path.to_string_lossy().into_owned(),
            )],
        )?;

        Ok((client, url, temp_dir))
    }

    pub async fn spawn_stdio_client_with_database() -> io::Result<(StdioClient, String, TempDir)> {
        let database = database_override()?;
        let url = database.url.clone();
        let temp_dir = database.temp_dir;
        let client = StdioClient::spawn(&url, None)?;
        Ok((client, url, temp_dir))
    }

    struct DatabaseOverride {
        temp_dir: TempDir,
        url: String,
    }

    pub struct StdioClient {
        _database_dir: Option<TempDir>,
        child: Child,
        initialized: bool,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    }

    pub struct StubServer {
        base_url: String,
        last_request_path: Arc<tokio::sync::Mutex<Option<String>>>,
        request_count: Arc<tokio::sync::Mutex<usize>>,
        shutdown: Option<oneshot::Sender<()>>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Tool {
        pub name: String,
    }

    impl StdioClient {
        fn spawn(database_url: &str, database_dir: Option<TempDir>) -> io::Result<Self> {
            Self::spawn_with_env(
                database_dir,
                &[(DATABASE_URL_ENV_VAR, database_url.to_string())],
            )
        }

        fn spawn_with_env(
            database_dir: Option<TempDir>,
            envs: &[(&str, String)],
        ) -> io::Result<Self> {
            let mut command = Command::new(env!("CARGO_BIN_EXE_agent_llm_mm"));
            for (key, value) in envs {
                command.env(key, value);
            }
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("missing child stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("missing child stdout"))?;

            Ok(Self {
                _database_dir: database_dir,
                child,
                initialized: false,
                stdin,
                stdout: BufReader::new(stdout),
            })
        }

        pub async fn list_all_tools(&mut self) -> io::Result<Vec<Tool>> {
            self.initialize()?;
            self.list_tools()
        }

        pub async fn call_tool(&mut self, name: &str, arguments: Value) -> io::Result<Value> {
            self.initialize()?;
            self.send(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": name,
                    "arguments": arguments
                }
            }))?;
            self.read_message()
        }

        fn initialize(&mut self) -> io::Result<()> {
            if self.initialized {
                return Ok(());
            }

            self.send(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-stdio-test",
                        "version": "0.1.0"
                    }
                }
            }))?;
            let _ = self.read_message()?;

            self.send(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }))?;
            self.initialized = true;
            Ok(())
        }

        fn list_tools(&mut self) -> io::Result<Vec<Tool>> {
            self.send(json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }))?;

            let message = self.read_message()?;
            let tools = message
                .get("result")
                .and_then(|result| result.get("tools"))
                .cloned()
                .ok_or_else(|| io::Error::other("missing tools in response"))?;

            serde_json::from_value(tools)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        }

        fn send(&mut self, payload: Value) -> io::Result<()> {
            let mut body = serde_json::to_vec(&payload)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            body.push(b'\n');
            self.stdin.write_all(&body)?;
            self.stdin.flush()
        }

        fn read_message(&mut self) -> io::Result<Value> {
            loop {
                let mut line = String::new();
                let bytes_read = self.stdout.read_line(&mut line)?;
                if bytes_read == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "child process closed stdout before sending an MCP message",
                    ));
                }

                let trimmed = line.trim();
                if !trimmed.starts_with('{') {
                    continue;
                }

                return serde_json::from_str(trimmed)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
            }
        }
    }

    impl Drop for StdioClient {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    impl StubServer {
        pub async fn spawn(status: u16, body: Value) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let address = listener.local_addr().expect("local addr");
            let base_url = format!("http://{address}");
            let last_request_path = Arc::new(tokio::sync::Mutex::new(None));
            let request_path = Arc::clone(&last_request_path);
            let request_count = Arc::new(tokio::sync::Mutex::new(0));
            let request_counter = Arc::clone(&request_count);
            let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
            let response_body = body.to_string();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = &mut shutdown_rx => break,
                        accept = listener.accept() => {
                            if let Ok((mut stream, _)) = accept {
                                let mut buffer = vec![0_u8; 16 * 1024];
                                let bytes_read = stream.read(&mut buffer).await.expect("read");
                                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                                let path = request
                                    .lines()
                                    .next()
                                    .and_then(|line| line.split_whitespace().nth(1))
                                    .map(str::to_string);
                                *request_path.lock().await = path;
                                *request_counter.lock().await += 1;

                                let status_text = match status {
                                    200 => "OK",
                                    503 => "Service Unavailable",
                                    _ => "Test Status",
                                };
                                let response = format!(
                                    "HTTP/1.1 {status} {status_text}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                                    response_body.len(),
                                    response_body
                                );
                                stream
                                    .write_all(response.as_bytes())
                                    .await
                                    .expect("write");
                            }
                        }
                    }
                }
            });

            Self {
                base_url,
                last_request_path,
                request_count,
                shutdown: Some(shutdown_tx),
            }
        }

        pub fn base_url(&self) -> String {
            self.base_url.clone()
        }

        pub async fn last_request_path(&self) -> Option<String> {
            self.last_request_path.lock().await.clone()
        }

        pub async fn request_count(&self) -> usize {
            *self.request_count.lock().await
        }
    }

    impl Drop for StubServer {
        fn drop(&mut self) {
            if let Some(shutdown) = self.shutdown.take() {
                let _ = shutdown.send(());
            }
        }
    }

    fn database_override() -> io::Result<DatabaseOverride> {
        let temp_dir = tempfile::tempdir()?;
        let database_path = temp_dir.path().join("agent-llm-mm.sqlite");
        Ok(DatabaseOverride {
            url: sqlite_url(&database_path),
            temp_dir,
        })
    }

    fn sqlite_url(path: &Path) -> String {
        format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
    }
}
