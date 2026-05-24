use serde::Serialize;

use crate::{
    domain::operation_log::{OperationLogKind, OperationLogStatus},
    interfaces,
    ports::{OperationLogQuery, OperationLogStore},
    support::config::{AppConfig, ModelProviderKind, ProviderMatrixEntry, TransportKind},
};

use super::remote_team::{
    RemoteTeamCapabilityInventory, RemoteTeamSecurityGateReport, remote_team_capability_inventory,
    remote_team_security_gate_report,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub transport: TransportKind,
    pub database_url: String,
    pub provider: ModelProviderKind,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub provider_matrix: Vec<DoctorProviderMatrixEntry>,
    pub dashboard_enabled: bool,
    pub dashboard_host: String,
    pub dashboard_port: u16,
    pub dashboard_base_path: String,
    pub dashboard_required: bool,
    pub daemon_enabled: bool,
    pub daemon_poll_interval_ms: u64,
    pub daemon_max_concurrent_tasks: u32,
    pub daemon_observe_only: DaemonObserveOnlyDiagnostics,
    pub remote_team_capability_inventory: RemoteTeamCapabilityInventory,
    pub remote_team_security_gates: RemoteTeamSecurityGateReport,
    pub auto_reflection_runtime_hooks: Vec<String>,
    pub self_revision_write_path: &'static str,
    pub status: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderSupportState {
    Supported,
    PlannedOnly,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorProviderMatrixEntry {
    pub provider: &'static str,
    pub support_state: ProviderSupportState,
    pub configurable: bool,
    pub adapter: &'static str,
    pub missing_implementation: &'static str,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DaemonObserveOnlyDiagnostics {
    pub mode: &'static str,
    pub local_only: bool,
    pub write_gate_approved: bool,
    pub writes_allowed: bool,
    pub remote_listener_enabled: bool,
    pub write_blockers: Vec<String>,
    pub remote_blockers: Vec<String>,
    pub data_sources: Vec<String>,
    pub trigger_candidates_observed: usize,
    pub trigger_candidates_suppressed: usize,
    pub cooldown_status: &'static str,
    pub in_flight_task_count: usize,
    pub read_errors: Vec<String>,
}

pub async fn run_doctor(config: AppConfig) -> anyhow::Result<DoctorReport> {
    config.validate().map_err(anyhow::Error::msg)?;

    let base_url = config.doctor_base_url();
    let model = config.doctor_model();

    let runtime = match config.transport {
        TransportKind::Stdio => interfaces::mcp::validate_stdio_runtime(&config).await?,
    };
    let daemon_observe_only = build_daemon_observe_only_diagnostics(&config, &runtime).await;
    let provider_matrix = build_provider_matrix(&config);
    let remote_team_capability_inventory = remote_team_capability_inventory();
    let remote_team_security_gates = remote_team_security_gate_report();

    Ok(DoctorReport {
        transport: config.transport,
        database_url: config.database_url,
        provider: config.model_provider,
        base_url,
        model,
        provider_matrix,
        dashboard_enabled: config.dashboard.enabled,
        dashboard_host: config.dashboard.host,
        dashboard_port: config.dashboard.port,
        dashboard_base_path: config.dashboard.base_path,
        dashboard_required: config.dashboard.required,
        daemon_enabled: config.daemon.enabled,
        daemon_poll_interval_ms: config.daemon.poll_interval_ms,
        daemon_max_concurrent_tasks: config.daemon.max_concurrent_tasks,
        daemon_observe_only,
        remote_team_capability_inventory,
        remote_team_security_gates,
        auto_reflection_runtime_hooks: interfaces::mcp::server::AUTO_REFLECTION_RUNTIME_HOOKS
            .iter()
            .map(|hook| hook.to_string())
            .collect(),
        self_revision_write_path: interfaces::mcp::server::SELF_REVISION_WRITE_PATH,
        status: "ok",
    })
}

fn build_provider_matrix(config: &AppConfig) -> Vec<DoctorProviderMatrixEntry> {
    AppConfig::provider_matrix()
        .into_iter()
        .map(|entry| doctor_provider_matrix_entry(config, entry))
        .collect()
}

fn doctor_provider_matrix_entry(
    config: &AppConfig,
    entry: ProviderMatrixEntry,
) -> DoctorProviderMatrixEntry {
    DoctorProviderMatrixEntry {
        provider: entry.provider,
        support_state: match entry.state {
            "supported" => ProviderSupportState::Supported,
            _ => ProviderSupportState::PlannedOnly,
        },
        configurable: entry.configurable,
        adapter: entry.adapter,
        missing_implementation: entry.missing_implementation,
        selected: entry.provider == config.model_provider.as_str(),
    }
}

async fn build_daemon_observe_only_diagnostics(
    config: &AppConfig,
    operation_log: &impl OperationLogStore,
) -> DaemonObserveOnlyDiagnostics {
    let mut read_errors = Vec::new();
    let (trigger_candidates_observed, trigger_candidates_suppressed) = if config.daemon.enabled {
        (
            count_trigger_candidates(operation_log, OperationLogStatus::Failed, &mut read_errors)
                .await,
            count_trigger_candidates(
                operation_log,
                OperationLogStatus::Suppressed,
                &mut read_errors,
            )
            .await,
        )
    } else {
        (0, 0)
    };

    DaemonObserveOnlyDiagnostics {
        mode: "observe_only",
        local_only: true,
        write_gate_approved: false,
        writes_allowed: false,
        remote_listener_enabled: false,
        write_blockers: vec![
            "daemon write gate is not approved; run_reflection remains the only durable write path"
                .to_string(),
            "observe-only diagnostics must not write identity, commitments, claims, events, or reflections".to_string(),
        ],
        remote_blockers: vec![
            "remote listener is blocked until auth, authorization, audit, rollback, and tenant isolation gates exist".to_string(),
            "remote/team mode is not implemented in the local MVP".to_string(),
        ],
        data_sources: vec!["daemon_config".to_string(), "operation_log".to_string()],
        trigger_candidates_observed,
        trigger_candidates_suppressed,
        cooldown_status: "observe_only",
        in_flight_task_count: 0,
        read_errors,
    }
}

async fn count_trigger_candidates(
    operation_log: &impl OperationLogStore,
    status: OperationLogStatus,
    read_errors: &mut Vec<String>,
) -> usize {
    let mut count = 0;
    for kind in [OperationLogKind::Tool, OperationLogKind::Trigger] {
        match operation_log
            .query_operations(OperationLogQuery {
                operation_kind: Some(kind.as_str().to_string()),
                status: Some(status.as_str().to_string()),
                limit: Some(25),
                ..Default::default()
            })
            .await
        {
            Ok(entries) => {
                count += entries.len();
            }
            Err(error) => read_errors.push(format!("operation_log:{}:{}", kind.as_str(), error)),
        }
    }
    count
}
