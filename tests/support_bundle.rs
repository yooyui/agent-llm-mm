use agent_llm_mm::{
    adapters::sqlite::SqliteStore,
    domain::operation_log::{ActorKind, OperationLogEntry, OperationLogKind, OperationLogStatus},
    ports::OperationLogStore,
    support::{
        config::{
            AppConfig, DashboardConfig, ModelConfig, ModelProviderKind, OpenAiCompatibleConfig,
            TransportKind,
        },
        support_bundle::{SupportBundleOptions, generate_support_bundle},
    },
};
use chrono::{Duration, Utc};
use serde_json::Value;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
use tempfile::tempdir;

#[tokio::test]
async fn support_bundle_generates_redacted_local_diagnostics() {
    let temp_dir = tempdir().expect("temp dir");
    let database_path = temp_dir.path().join("support-bundle.sqlite");
    let database_url = sqlite_url(database_path.clone());
    let store = SqliteStore::bootstrap(&database_url)
        .await
        .expect("store should bootstrap");
    let secret = "sk-support-secret-12345";
    let base_url_user = "bundle-user";
    let base_url_secret = "base-url-token-12345";
    let base_url_query_secret = "base-url-query-secret";
    let now = Utc::now();

    for index in 0..30 {
        store
            .append_operation(OperationLogEntry {
                operation_id: format!("op-{index:02}"),
                occurred_at: now + Duration::seconds(index),
                namespace: Some("self".to_string()),
                actor_kind: ActorKind::System,
                actor_id: "mcp-stdio".to_string(),
                entrypoint: "ingest_interaction".to_string(),
                operation_kind: OperationLogKind::Tool,
                status: OperationLogStatus::Ok,
                correlation_id: Some(format!("corr-{index:02}")),
                request_summary_json: Some(format!(r#"{{"secret":"{secret}"}}"#)),
                response_summary_json: Some(r#"{"event_id":"event-1"}"#.to_string()),
                diagnostic_summary_json: None,
                redaction_version: 1,
            })
            .await
            .expect("operation append should succeed");
    }
    let identity_count_before = table_count(&database_url, "identity_claims").await;
    let commitment_count_before = table_count(&database_url, "commitments").await;

    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url: database_url.clone(),
        model_provider: ModelProviderKind::OpenAiCompatible,
        model_config: ModelConfig::OpenAiCompatible(OpenAiCompatibleConfig {
            base_url: format!(
                "https://{base_url_user}:{base_url_secret}@api.example.test/v1?token={base_url_query_secret}"
            ),
            api_key: secret.to_string(),
            model: "gpt-4o-mini".to_string(),
            timeout_ms: 30_000,
        }),
        dashboard: DashboardConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 8787,
            base_path: "/agent-llm-mm".to_string(),
            event_capacity: 2000,
            sse_enabled: true,
            open_browser: false,
            required: true,
        },
        ..Default::default()
    };
    let output_dir = temp_dir.path().join("bundle");

    generate_support_bundle(SupportBundleOptions {
        config,
        output_dir: output_dir.clone(),
        config_path: None,
        project_root: temp_dir.path().to_path_buf(),
    })
    .await
    .expect("support bundle should generate");

    assert_eq!(
        table_count(&database_url, "identity_claims").await,
        identity_count_before,
        "support bundle generation must not write identity rows"
    );
    assert_eq!(
        table_count(&database_url, "commitments").await,
        commitment_count_before,
        "support bundle generation must not write commitment rows"
    );

    for file in [
        "manifest.json",
        "doctor.json",
        "config-shape.json",
        "operation-summaries.json",
        "release-metadata.json",
        "product-smoke-summary.json",
    ] {
        assert!(
            output_dir.join(file).is_file(),
            "support bundle should include {file}"
        );
    }

    let all_bundle_text = fs::read_dir(&output_dir)
        .expect("bundle dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| fs::read_to_string(entry.path()).expect("bundle file text"))
        .collect::<Vec<_>>()
        .join("\n");
    let lower_bundle_text = all_bundle_text.to_lowercase();

    assert!(!all_bundle_text.contains(secret));
    assert!(!all_bundle_text.contains(base_url_user));
    assert!(!all_bundle_text.contains(base_url_secret));
    assert!(!all_bundle_text.contains(base_url_query_secret));
    assert!(!all_bundle_text.contains(database_path.to_string_lossy().as_ref()));
    assert!(!lower_bundle_text.contains("api_key"));
    assert!(!all_bundle_text.contains("Authorization"));
    assert!(!all_bundle_text.contains("Bearer"));
    assert!(
        fs::read_dir(&output_dir)
            .expect("bundle entries")
            .filter_map(|entry| entry.ok())
            .all(|entry| entry.path().extension().and_then(|ext| ext.to_str()) != Some("sqlite")),
        "support bundle must not include full sqlite databases by default"
    );

    let doctor = read_json(output_dir.join("doctor.json"));
    assert_eq!(doctor["status"], "config-shape-ok");
    assert_eq!(doctor["database_url_shape"], "sqlite://<local-path>");
    assert_eq!(doctor["self_revision_write_path"], "run_reflection");
    assert_eq!(doctor["runtime_bootstrap_performed"], false);

    let manifest = read_json(output_dir.join("manifest.json"));
    assert_eq!(manifest["local_only"], true);
    assert_eq!(manifest["upload_performed"], false);
    assert!(
        manifest["excluded_by_default"]
            .as_array()
            .expect("manifest excluded list")
            .iter()
            .any(|entry| entry == "provider url userinfo and query values")
    );

    let config_shape = read_json(output_dir.join("config-shape.json"));
    assert_eq!(config_shape["model"]["provider"], "openai-compatible");
    assert_eq!(
        config_shape["model"]["base_url"],
        "https://api.example.test/v1"
    );
    assert_eq!(config_shape["model"]["credential_configured"], true);
    assert_eq!(config_shape["dashboard"]["enabled"], true);
    assert_eq!(config_shape["database_url_shape"], "sqlite://<local-path>");

    let operation_summaries = read_json(output_dir.join("operation-summaries.json"));
    let entries = operation_summaries["entries"]
        .as_array()
        .expect("operation entries array");
    assert_eq!(entries.len(), 25);
    assert_eq!(operation_summaries["limit"], 25);
    assert_eq!(operation_summaries["available"], true);
    assert_eq!(operation_summaries["unavailable_reason"], Value::Null);
    assert!(entries.iter().all(|entry| entry["read_only"] == true));
}

#[tokio::test]
async fn support_bundle_does_not_create_or_bootstrap_missing_database() {
    let temp_dir = tempdir().expect("temp dir");
    let database_path = temp_dir.path().join("missing.sqlite");
    let output_dir = temp_dir.path().join("bundle");
    let config = AppConfig {
        database_url: sqlite_url(database_path.clone()),
        ..Default::default()
    };

    generate_support_bundle(SupportBundleOptions {
        config,
        output_dir: output_dir.clone(),
        config_path: None,
        project_root: temp_dir.path().to_path_buf(),
    })
    .await
    .expect("support bundle should generate without bootstrapping db");

    assert!(
        !database_path.exists(),
        "support bundle generation must not create or migrate the sqlite database"
    );

    let doctor = read_json(output_dir.join("doctor.json"));
    assert_eq!(doctor["runtime_bootstrap_performed"], false);
    assert_eq!(doctor["self_revision_write_path"], "run_reflection");

    let operation_summaries = read_json(output_dir.join("operation-summaries.json"));
    assert_eq!(operation_summaries["available"], false);
    assert_eq!(
        operation_summaries["unavailable_reason"],
        "sqlite database file not found"
    );
    assert_eq!(
        operation_summaries["entries"]
            .as_array()
            .expect("entries array")
            .len(),
        0
    );
}

#[tokio::test]
async fn support_bundle_refuses_non_empty_output_directory() {
    let temp_dir = tempdir().expect("temp dir");
    let output_dir = temp_dir.path().join("bundle");
    fs::create_dir_all(&output_dir).expect("output dir");
    fs::write(
        output_dir.join("agent-llm-mm.local.toml"),
        "api_key = \"sk-existing\"",
    )
    .expect("pre-existing file");

    let result = generate_support_bundle(SupportBundleOptions {
        config: AppConfig::default(),
        output_dir: output_dir.clone(),
        config_path: None,
        project_root: temp_dir.path().to_path_buf(),
    })
    .await;

    let error = result.expect_err("non-empty output dirs must be rejected");
    assert!(
        error.to_string().contains("must be empty"),
        "unexpected error: {error:#}"
    );
    assert!(
        !output_dir.join("manifest.json").exists(),
        "generator must fail before writing support bundle artifacts"
    );
}

#[test]
fn support_bundle_script_uses_dedicated_generator_binary() {
    let script =
        fs::read_to_string("scripts/generate-support-bundle.sh").expect("script should exist");
    let mode = fs::metadata("scripts/generate-support-bundle.sh")
        .expect("script metadata")
        .permissions()
        .mode();

    assert!(script.contains("cargo run --quiet --bin generate_support_bundle --"));
    assert!(script.contains("usage: ./scripts/generate-support-bundle.sh"));
    assert!(script.contains("mkdir -p \"${output_parent}\""));
    assert_ne!(mode & 0o111, 0, "script should be directly executable");
}

fn sqlite_url(path: PathBuf) -> String {
    format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
}

fn read_json(path: PathBuf) -> Value {
    serde_json::from_slice(&fs::read(path).expect("json file")).expect("valid json")
}

async fn table_count(database_url: &str, table: &str) -> i64 {
    let pool = sqlx::SqlitePool::connect(database_url)
        .await
        .expect("sqlite pool");
    let sql = format!("SELECT COUNT(*) FROM {table}");
    sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(&pool)
        .await
        .expect("table count")
}
