use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::Serialize;
use sqlx::{
    Row,
    sqlite::{SqliteConnectOptions, SqlitePool},
};

use crate::{
    interfaces,
    support::config::{AppConfig, ModelConfig},
};

const OPERATION_SUMMARY_LIMIT: usize = 25;

#[derive(Debug, Clone)]
pub struct SupportBundleOptions {
    pub config: AppConfig,
    pub output_dir: PathBuf,
    pub config_path: Option<PathBuf>,
    pub project_root: PathBuf,
}

#[derive(Debug, Serialize)]
struct SafeDoctorReport {
    transport: String,
    database_url_shape: String,
    provider: String,
    base_url: Option<String>,
    model: Option<String>,
    dashboard_enabled: bool,
    dashboard_host: String,
    dashboard_port: u16,
    dashboard_base_path: String,
    dashboard_required: bool,
    daemon_enabled: bool,
    daemon_poll_interval_ms: u64,
    daemon_max_concurrent_tasks: u32,
    auto_reflection_runtime_hooks: Vec<String>,
    self_revision_write_path: &'static str,
    runtime_bootstrap_performed: bool,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ConfigShape {
    transport: String,
    database_url_shape: String,
    model: ModelShape,
    dashboard: DashboardShape,
    daemon: DaemonShape,
    config_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModelShape {
    provider: String,
    base_url: Option<String>,
    model: Option<String>,
    timeout_ms: Option<u64>,
    credential_configured: bool,
}

#[derive(Debug, Serialize)]
struct DashboardShape {
    enabled: bool,
    host: String,
    port: u16,
    base_path: String,
    event_capacity: usize,
    sse_enabled: bool,
    open_browser: bool,
    required: bool,
}

#[derive(Debug, Serialize)]
struct DaemonShape {
    enabled: bool,
    poll_interval_ms: u64,
    max_concurrent_tasks: u32,
}

#[derive(Debug, Serialize)]
struct OperationSummaries {
    limit: usize,
    available: bool,
    unavailable_reason: Option<&'static str>,
    entries: Vec<OperationSummary>,
}

#[derive(Debug, Serialize)]
struct OperationSummary {
    operation_id: String,
    occurred_at: String,
    namespace: Option<String>,
    entrypoint: String,
    kind: String,
    status: String,
    correlation_id: Option<String>,
    read_only: bool,
}

#[derive(Debug, Serialize)]
struct ReleaseMetadata {
    generated_at: String,
    git_commit: Option<String>,
    git_branch: Option<String>,
    platform: String,
    rustc_version: Option<String>,
    verification_commands: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ProductSmokeSummary {
    required_artifacts: Vec<&'static str>,
    latest_artifacts_present: bool,
    self_revision_write_path_expected: &'static str,
}

#[derive(Debug, Serialize)]
struct Manifest {
    bundle_format: &'static str,
    generated_at: String,
    local_only: bool,
    upload_performed: bool,
    excluded_by_default: Vec<&'static str>,
    files: Vec<&'static str>,
}

pub async fn generate_support_bundle(options: SupportBundleOptions) -> Result<()> {
    let SupportBundleOptions {
        config,
        output_dir,
        config_path,
        project_root,
    } = options;

    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "failed to create support bundle directory {}",
            output_dir.display()
        )
    })?;
    ensure_empty_output_dir(&output_dir)?;

    config.validate().map_err(anyhow::Error::msg)?;

    let operations = query_operation_summaries(&config.database_url).await;
    let generated_at = Utc::now().to_rfc3339();

    write_json(
        &output_dir.join("doctor.json"),
        &safe_doctor_report(&config),
    )?;
    write_json(
        &output_dir.join("config-shape.json"),
        &config_shape(&config, config_path.as_deref()),
    )?;
    write_json(&output_dir.join("operation-summaries.json"), &operations)?;
    write_json(
        &output_dir.join("release-metadata.json"),
        &release_metadata(&project_root, &generated_at),
    )?;
    write_json(
        &output_dir.join("product-smoke-summary.json"),
        &product_smoke_summary(&project_root),
    )?;
    write_json(&output_dir.join("manifest.json"), &manifest(&generated_at))?;

    Ok(())
}

fn ensure_empty_output_dir(output_dir: &Path) -> Result<()> {
    let mut entries = fs::read_dir(output_dir).with_context(|| {
        format!(
            "failed to inspect support bundle directory {}",
            output_dir.display()
        )
    })?;
    if entries.next().transpose()?.is_some() {
        anyhow::bail!(
            "support bundle output directory must be empty: {}",
            output_dir.display()
        );
    }

    Ok(())
}

fn safe_doctor_report(config: &AppConfig) -> SafeDoctorReport {
    let model = model_shape(config);

    SafeDoctorReport {
        transport: serde_name(&config.transport),
        database_url_shape: database_url_shape(&config.database_url),
        provider: serde_name(&config.model_provider),
        base_url: model.base_url,
        model: model.model,
        dashboard_enabled: config.dashboard.enabled,
        dashboard_host: config.dashboard.host.clone(),
        dashboard_port: config.dashboard.port,
        dashboard_base_path: config.dashboard.base_path.clone(),
        dashboard_required: config.dashboard.required,
        daemon_enabled: config.daemon.enabled,
        daemon_poll_interval_ms: config.daemon.poll_interval_ms,
        daemon_max_concurrent_tasks: config.daemon.max_concurrent_tasks,
        auto_reflection_runtime_hooks: interfaces::mcp::server::AUTO_REFLECTION_RUNTIME_HOOKS
            .iter()
            .map(|hook| hook.to_string())
            .collect(),
        self_revision_write_path: interfaces::mcp::server::SELF_REVISION_WRITE_PATH,
        runtime_bootstrap_performed: false,
        status: "config-shape-ok",
    }
}

fn config_shape(config: &AppConfig, config_path: Option<&Path>) -> ConfigShape {
    let model = model_shape(config);

    ConfigShape {
        transport: serde_name(&config.transport),
        database_url_shape: database_url_shape(&config.database_url),
        model,
        dashboard: DashboardShape {
            enabled: config.dashboard.enabled,
            host: config.dashboard.host.clone(),
            port: config.dashboard.port,
            base_path: config.dashboard.base_path.clone(),
            event_capacity: config.dashboard.event_capacity,
            sse_enabled: config.dashboard.sse_enabled,
            open_browser: config.dashboard.open_browser,
            required: config.dashboard.required,
        },
        daemon: DaemonShape {
            enabled: config.daemon.enabled,
            poll_interval_ms: config.daemon.poll_interval_ms,
            max_concurrent_tasks: config.daemon.max_concurrent_tasks,
        },
        config_path: config_path.map(redact_path),
    }
}

fn model_shape(config: &AppConfig) -> ModelShape {
    match &config.model_config {
        ModelConfig::Mock => ModelShape {
            provider: serde_name(&config.model_provider),
            base_url: None,
            model: None,
            timeout_ms: None,
            credential_configured: false,
        },
        ModelConfig::OpenAiCompatible(provider) => ModelShape {
            provider: serde_name(&config.model_provider),
            base_url: Some(base_url_shape(&provider.base_url)),
            model: Some(provider.model.clone()),
            timeout_ms: Some(provider.timeout_ms),
            credential_configured: !provider.api_key.trim().is_empty(),
        },
    }
}

async fn query_operation_summaries(database_url: &str) -> OperationSummaries {
    if let Some(path) = sqlite_database_file_path(database_url)
        && !path.is_file()
    {
        return unavailable_operation_summaries("sqlite database file not found");
    }

    match query_operation_summaries_read_only(database_url).await {
        Ok(entries) => OperationSummaries {
            limit: OPERATION_SUMMARY_LIMIT,
            available: true,
            unavailable_reason: None,
            entries,
        },
        Err(_) => unavailable_operation_summaries("read-only operation-log query unavailable"),
    }
}

async fn query_operation_summaries_read_only(database_url: &str) -> Result<Vec<OperationSummary>> {
    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|error| anyhow!(error.to_string()))?
        .read_only(true)
        .create_if_missing(false)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(options).await?;

    let table_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'operation_log'",
    )
    .fetch_one(&pool)
    .await?;
    if table_count == 0 {
        return Err(anyhow!("operation_log table is not present"));
    }

    let limit = i64::try_from(OPERATION_SUMMARY_LIMIT)
        .context("operation summary limit exceeds sqlite i64 range")?;
    let rows = sqlx::query(
        "SELECT operation_id, occurred_at, namespace, entrypoint, operation_kind, status, correlation_id FROM operation_log ORDER BY occurred_at DESC, operation_id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|entry| OperationSummary {
            operation_id: entry.get("operation_id"),
            occurred_at: entry.get("occurred_at"),
            namespace: entry.get("namespace"),
            entrypoint: entry.get("entrypoint"),
            kind: entry.get("operation_kind"),
            status: entry.get("status"),
            correlation_id: entry.get("correlation_id"),
            read_only: true,
        })
        .collect())
}

fn unavailable_operation_summaries(reason: &'static str) -> OperationSummaries {
    OperationSummaries {
        limit: OPERATION_SUMMARY_LIMIT,
        available: false,
        unavailable_reason: Some(reason),
        entries: Vec::new(),
    }
}

fn release_metadata(project_root: &Path, generated_at: &str) -> ReleaseMetadata {
    ReleaseMetadata {
        generated_at: generated_at.to_string(),
        git_commit: command_output(project_root, "git", &["rev-parse", "--short", "HEAD"]),
        git_branch: command_output(project_root, "git", &["branch", "--show-current"]),
        platform: std::env::consts::OS.to_string(),
        rustc_version: command_output(project_root, "rustc", &["--version"]),
        verification_commands: vec![
            "cargo fmt --check",
            "git diff --check",
            "cargo clippy --all-targets --all-features -- -D warnings",
            "cargo test",
            "./scripts/agent-llm-mm.sh doctor",
            "bash scripts/product-smoke-local.sh",
        ],
    }
}

fn product_smoke_summary(project_root: &Path) -> ProductSmokeSummary {
    let latest = project_root.join("target/reports/self-revision-demo/latest");
    let required_artifacts = vec![
        "doctor.json",
        "snapshot-before.json",
        "snapshot-after.json",
        "decision-before.json",
        "decision-after.json",
        "timeline.json",
        "sqlite-summary.json",
        "report.md",
    ];
    let latest_artifacts_present = required_artifacts
        .iter()
        .all(|artifact| latest.join(artifact).is_file());

    ProductSmokeSummary {
        required_artifacts,
        latest_artifacts_present,
        self_revision_write_path_expected: "run_reflection",
    }
}

fn manifest(generated_at: &str) -> Manifest {
    Manifest {
        bundle_format: "agent-llm-mm-local-alpha-support-bundle-v1",
        generated_at: generated_at.to_string(),
        local_only: true,
        upload_performed: false,
        excluded_by_default: vec![
            "api keys",
            "authorization headers",
            "bearer values",
            "raw provider payloads",
            "full sqlite databases",
            "unredacted toml files",
            "provider url userinfo and query values",
            "ssh keys",
            "cookies",
            "browser session data",
        ],
        files: vec![
            "manifest.json",
            "doctor.json",
            "config-shape.json",
            "operation-summaries.json",
            "release-metadata.json",
            "product-smoke-summary.json",
        ],
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn database_url_shape(database_url: &str) -> String {
    if database_url.starts_with("sqlite://") {
        "sqlite://<local-path>".to_string()
    } else {
        "<redacted>".to_string()
    }
}

fn sqlite_database_file_path(database_url: &str) -> Option<PathBuf> {
    let path = database_url.strip_prefix("sqlite://")?;
    let path = path.split_once('?').map_or(path, |(path, _)| path);
    if path.is_empty() || path == ":memory:" {
        return None;
    }

    #[cfg(windows)]
    let path = normalize_windows_sqlite_path(path);

    #[cfg(not(windows))]
    let path = path.to_string();

    Some(PathBuf::from(path))
}

#[cfg(windows)]
fn normalize_windows_sqlite_path(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':' {
        return path[1..].to_string();
    }

    path.to_string()
}

fn base_url_shape(base_url: &str) -> String {
    let Ok(parsed) = reqwest::Url::parse(base_url) else {
        return "<redacted>".to_string();
    };
    let Some(host) = parsed.host_str() else {
        return "<redacted>".to_string();
    };

    let mut shaped = format!("{}://{}", parsed.scheme(), host);
    if let Some(port) = parsed.port() {
        shaped.push(':');
        shaped.push_str(&port.to_string());
    }
    shaped.push_str(parsed.path());
    shaped
}

fn redact_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("<local-path>/{name}"))
        .unwrap_or_else(|| "<local-path>".to_string())
}

fn serde_name(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn command_output(cwd: &Path, command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
