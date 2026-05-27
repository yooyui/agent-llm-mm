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
    pub system_layer_report: SystemLayerReport,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SystemLayerReport {
    pub read_only: bool,
    pub writes_performed: bool,
    pub layers: Vec<SystemLayerEntry>,
    pub physics_principles: Vec<PhysicsPrincipleMapping>,
    pub dependency_rules: Vec<SystemDependencyRule>,
    pub phase_coverage: Vec<SystemPhaseCoverage>,
    pub blockers: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SystemLayerEntry {
    pub name: String,
    pub status: String,
    pub responsibility: String,
    pub anchors: Vec<String>,
    pub writes_allowed: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PhysicsPrincipleMapping {
    pub principle: String,
    pub engineering_interpretation: String,
    pub structural_rule: String,
    pub implementation_anchor: String,
    pub report_only: bool,
    pub grants_capability: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SystemDependencyRule {
    pub name: String,
    pub rule: String,
    pub enforced_as: String,
    pub verification: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SystemPhaseCoverage {
    pub phase: u8,
    pub name: String,
    pub status: String,
    pub implementation_mode: String,
    pub current_anchor: Vec<String>,
    pub blocked_items: Vec<String>,
    pub verification: Vec<String>,
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
    let system_layer_report =
        build_system_layer_report(&daemon_observe_only, &remote_team_security_gates);

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
        system_layer_report,
        auto_reflection_runtime_hooks: interfaces::mcp::server::AUTO_REFLECTION_RUNTIME_HOOKS
            .iter()
            .map(|hook| hook.to_string())
            .collect(),
        self_revision_write_path: interfaces::mcp::server::SELF_REVISION_WRITE_PATH,
        status: "ok",
    })
}

fn build_system_layer_report(
    daemon_observe_only: &DaemonObserveOnlyDiagnostics,
    remote_team_security_gates: &RemoteTeamSecurityGateReport,
) -> SystemLayerReport {
    let security_gate_names = remote_team_security_gates.blocked_gate_names().join(", ");
    let remote_writes_blocker = if remote_team_security_gates.remote_writes_allowed {
        "remote/team writes remain outside this read-only report".to_string()
    } else {
        format!("security/auth gates block remote/team writes: {security_gate_names}")
    };
    let daemon_writes_blocker = if daemon_observe_only.writes_allowed {
        "daemon writes remain outside this read-only report".to_string()
    } else {
        "daemon writes are blocked; observe-only diagnostics do not create write authority"
            .to_string()
    };

    SystemLayerReport {
        read_only: true,
        writes_performed: false,
        layers: vec![
            system_layer_entry(
                "substrate",
                "partial",
                "local process, config, SQLite storage, migrations, and platform entrypoints",
                ["scripts/agent-llm-mm.sh", "scripts/agent-llm-mm.ps1"],
                [
                    "Local Alpha still needs fresh product smoke, support bundle, and release decision evidence",
                    "fresh-machine evidence is not proven by local simulation",
                    "Windows parity runtime evidence is still missing",
                ],
            ),
            system_layer_entry(
                "signal",
                "implemented",
                "raw local observations, operation log metadata, trigger candidates, and bounded evidence handles",
                ["ingest_interaction", "operation_log"],
                ["richer signal ranking and widening remain outside this report"],
            ),
            system_layer_entry(
                "memory",
                "partial",
                "derived claims, snapshots, episode projections, and read-only memory-layer labels",
                ["build_self_snapshot", "memory_layer_projection"],
                [
                    "memory layering is partial; procedural memory and durable new layer writes are not implemented",
                    "future memory layers need migration, lifecycle, and evidence-link gates",
                ],
            ),
            system_layer_entry(
                "policy",
                "implemented",
                "local governance rules, product wording guardrails, commitment gate, and remote/team security gates",
                ["decide_with_snapshot", "product_wording_guard"],
                [remote_writes_blocker.as_str()],
            ),
            system_layer_entry(
                "control_loop",
                "partial",
                "local feedback hooks that observe, decide, suppress, cool down, and propose",
                ["auto_reflect_if_needed", "trigger_ledger"],
                [daemon_writes_blocker.as_str()],
            ),
            system_layer_entry(
                "actuator",
                "implemented",
                "durable semantic write path for governed identity, commitment, and reflection updates",
                [interfaces::mcp::server::SELF_REVISION_WRITE_PATH],
                [
                    "this read-only report does not allow writes",
                    "future daemon writes require a separate architecture decision, migration, and rollback gate",
                ],
            ),
            system_layer_entry(
                "interface",
                "partial",
                "MCP stdio, local dashboard, support bundle, doctor output, and release scripts",
                ["mcp_stdio", "local_dashboard", "support_bundle", "doctor"],
                [
                    "remote/team interfaces are blocked until product and security gates exist",
                    "write-capable interface behavior must pass through run_reflection or a later approved write-path decision",
                ],
            ),
            system_layer_entry(
                "release_boundary",
                "partial",
                "release evidence, platform parity, compatibility, product wording, and human release decision gates",
                [
                    "local_alpha_evidence_summary",
                    "release_gate",
                    "status_sync_check",
                ],
                [
                    "Local Alpha is not complete without fresh evidence and a human release decision",
                    "Windows parity and fresh-machine evidence remain open",
                ],
            ),
        ],
        physics_principles: physics_principle_mappings(),
        dependency_rules: system_dependency_rules(),
        phase_coverage: system_phase_coverage(),
        blockers: vec![
            "Local Alpha full release gate still needs fresh evidence and human decision".to_string(),
            "Windows parity runtime evidence is missing".to_string(),
            "fresh-machine evidence is missing; local simulation is not enough".to_string(),
            "remote/team behavior is blocked and not implemented".to_string(),
            daemon_writes_blocker,
            remote_writes_blocker,
            "memory layering remains partial; procedural memory and durable new layer writes are not implemented".to_string(),
        ],
        non_claims: vec![
            "not a physics solver".to_string(),
            "not a constraint optimizer".to_string(),
            "not a physical controller".to_string(),
            "not scientific validation evidence".to_string(),
            "not complete multi-layer cognition".to_string(),
            "not a remote/team product".to_string(),
            "not Local Alpha, Beta, GA, or production-ready certification".to_string(),
        ],
    }
}

fn system_layer_entry<const A: usize, const B: usize>(
    name: &str,
    status: &str,
    responsibility: &str,
    anchors: [&str; A],
    blockers: [&str; B],
) -> SystemLayerEntry {
    SystemLayerEntry {
        name: name.to_string(),
        status: status.to_string(),
        responsibility: responsibility.to_string(),
        anchors: anchors.iter().map(|anchor| anchor.to_string()).collect(),
        writes_allowed: false,
        blockers: blockers.iter().map(|blocker| blocker.to_string()).collect(),
    }
}

fn physics_principle_mappings() -> Vec<PhysicsPrincipleMapping> {
    vec![
        physics_principle_mapping(
            "causality",
            "effects must have traceable causes",
            "semantic writes, gate status changes, and release claims must link to evidence or a human decision",
            "run_reflection audit and release evidence gates",
        ),
        physics_principle_mapping(
            "conservation",
            "state cannot appear from nowhere",
            "planned or simulated capabilities stay blocked until code, tests, docs, and fresh evidence exist",
            "follow-up reality gates and status-sync-check",
        ),
        physics_principle_mapping(
            "arrow_of_time",
            "release evidence is ordered and dated",
            "stale MVP evidence cannot certify later Local Alpha, Beta, or GA claims",
            "local_alpha_evidence_summary and release decision artifacts",
        ),
        physics_principle_mapping(
            "locality",
            "interactions happen through bounded local interfaces",
            "the MCP stdio core stays local and new surfaces must remain bounded",
            "MCP stdio, local dashboard, support bundle, and doctor",
        ),
        physics_principle_mapping(
            "feedback_control",
            "stable systems separate sensors, controllers, and actuators",
            "observe-only daemon diagnostics must not become write authority",
            "daemon_observe_only and run_reflection boundary",
        ),
        physics_principle_mapping(
            "entropy_increase",
            "complexity grows unless bounded",
            "status labels, drift checks, and small slices prevent plan and reality divergence",
            "status_sync_check and product wording guard",
        ),
        physics_principle_mapping(
            "energy_budget",
            "work is constrained by available evidence and verification budget",
            "prefer read-only reports and diagnostics before increasing write power or distribution scope",
            "system_layer_report",
        ),
        physics_principle_mapping(
            "boundary_conditions",
            "system behavior depends on external constraints",
            "remote/team and security-sensitive work stays blocked until auth, authorization, audit, rate limit, tenant isolation, rollback, and threat-model gates exist",
            "remote_team_security_gates",
        ),
    ]
}

fn physics_principle_mapping(
    principle: &str,
    engineering_interpretation: &str,
    structural_rule: &str,
    implementation_anchor: &str,
) -> PhysicsPrincipleMapping {
    PhysicsPrincipleMapping {
        principle: principle.to_string(),
        engineering_interpretation: engineering_interpretation.to_string(),
        structural_rule: structural_rule.to_string(),
        implementation_anchor: implementation_anchor.to_string(),
        report_only: true,
        grants_capability: false,
    }
}

fn system_dependency_rules() -> Vec<SystemDependencyRule> {
    vec![
        system_dependency_rule(
            "actuator_has_no_dashboard_or_release_dependency",
            "Actuator must not depend on dashboard or release tooling.",
            ["cargo test --test product_completion_read_models -v"],
        ),
        system_dependency_rule(
            "write_capable_interface_requires_run_reflection_or_adr",
            "Write-capable interfaces must pass through run_reflection or a later approved write-path ADR.",
            ["./scripts/agent-llm-mm.sh doctor"],
        ),
        system_dependency_rule(
            "observe_only_daemon_must_not_call_actuator",
            "Observe-only daemon diagnostics may read candidates but must not call the actuator.",
            ["cargo test --test daemon_config -v"],
        ),
        system_dependency_rule(
            "release_boundary_cannot_generate_external_evidence",
            "Release tooling can summarize evidence but cannot fabricate Windows runtime parity, real fresh-machine evidence, remote/team readiness, or a human release decision.",
            ["./scripts/status-sync-check.sh"],
        ),
        system_dependency_rule(
            "memory_writes_require_migration_and_lifecycle_gates",
            "Future durable memory layers require migration, lifecycle, evidence-link, rollback, and write-path review gates.",
            ["cargo test --test sqlite_backup_restore -v"],
        ),
    ]
}

fn system_dependency_rule<const V: usize>(
    name: &str,
    rule: &str,
    verification: [&str; V],
) -> SystemDependencyRule {
    SystemDependencyRule {
        name: name.to_string(),
        rule: rule.to_string(),
        enforced_as: "read-only-boundary".to_string(),
        verification: verification
            .iter()
            .map(|command| command.to_string())
            .collect(),
    }
}

fn system_phase_coverage() -> Vec<SystemPhaseCoverage> {
    vec![
        system_phase(
            0,
            "Preserve Baseline And Review Gates",
            "implemented",
            "local read-only drift and boundary checks",
            ["status_sync_check", "doctor", "product_wording_guard"],
            ["fresh evidence must still be rerun before new release claims"],
            [
                "git diff --check",
                "./scripts/status-sync-check.sh",
                "./scripts/agent-llm-mm.sh doctor",
            ],
        ),
        system_phase(
            1,
            "Read-Only Architecture Reports",
            "implemented",
            "doctor.system_layer_report exposes layer, law, dependency, phase, and non-claim boundaries",
            ["doctor.system_layer_report"],
            [
                "does not grant daemon writes, remote/team behavior, provider adapters, or durable memory layer writes",
            ],
            [
                "cargo test --test product_completion_read_models -v",
                "./scripts/agent-llm-mm.sh doctor",
            ],
        ),
        system_phase(
            2,
            "Local Alpha Evidence Completion",
            "partial",
            "local evidence refresh and source-only decision templates",
            [
                "local_alpha_evidence_summary",
                "product_readiness_check",
                "release_decision_local",
            ],
            [
                "real fresh-machine evidence is missing",
                "Windows runtime parity evidence is missing",
                "human release decision is missing",
            ],
            [
                "./scripts/local-alpha-release-gate-refresh.sh",
                "./scripts/product-readiness-check.sh <candidate-name>",
            ],
        ),
        system_phase(
            3,
            "Signal And Evidence Semantics",
            "partial",
            "bounded no-widening read-only relation projection",
            ["evidence_relation_report", "failure_modes"],
            ["full ranking and weighting engine is not implemented"],
            [
                "cargo test --test evidence_query_dto -v",
                "cargo test --test failure_modes -v",
            ],
        ),
        system_phase(
            4,
            "Memory Layering, Still Local And Gated",
            "partial",
            "read-only episode and layered memory projections",
            ["episode_summary_projection", "memory_layer_projection"],
            [
                "procedural memory is not implemented",
                "durable self-model writes require migration, lifecycle, and rollback gates",
            ],
            [
                "cargo test --test product_completion_read_models -v",
                "cargo test --test sqlite_backup_restore -v",
            ],
        ),
        system_phase(
            5,
            "Observe-Only Daemon Stabilization",
            "partial",
            "observe-only diagnostics with daemon writes and remote listener closed",
            ["daemon_observe_only"],
            [
                "daemon-triggered writes are blocked",
                "daemon loop as a connected product service is blocked",
                "all-entry automatic self-reflection is blocked",
            ],
            [
                "cargo test --test daemon_config -v",
                "./scripts/agent-llm-mm.sh doctor",
            ],
        ),
        system_phase(
            6,
            "Provider Expansion",
            "planning-gate",
            "planned-only provider rows remain rejected until one-provider-at-a-time implementation",
            ["provider_matrix"],
            [
                "provider adapter expansion is not implemented",
                "Azure OpenAI, OpenRouter, and local providers remain planned-only",
            ],
            [
                "cargo test --test provider_config -v",
                "./scripts/agent-llm-mm.sh doctor",
            ],
        ),
        system_phase(
            7,
            "Remote/Team/Security Foundation",
            "planning-gate",
            "blocked inventory and unsatisfied security gate contracts",
            [
                "remote_team_capability_inventory",
                "remote_team_security_gates",
            ],
            [
                "remote write admin is blocked",
                "team shared memory is blocked",
                "tenant isolation is not implemented",
                "auth, authorization, audit, rate limit, and rollback are not implemented",
            ],
            [
                "cargo test --test product_completion_read_models -v",
                "./scripts/product-readiness-check.sh <candidate-name>",
            ],
        ),
        system_phase(
            8,
            "Release Engineering And Beta/GA Readiness",
            "partial",
            "local source-only soak and candidate evidence diagnostics",
            ["release_soak_local", "product_readiness_check"],
            [
                "installer is not implemented",
                "binary package and service manager are not implemented",
                "compatibility matrix automation is not implemented",
                "Beta/GA claims are blocked",
            ],
            [
                "bash -n scripts/release-soak-local.sh",
                "cargo test --test local_alpha_release_evidence release_soak -v",
            ],
        ),
    ]
}

fn system_phase<const A: usize, const B: usize, const V: usize>(
    phase: u8,
    name: &str,
    status: &str,
    implementation_mode: &str,
    current_anchor: [&str; A],
    blocked_items: [&str; B],
    verification: [&str; V],
) -> SystemPhaseCoverage {
    SystemPhaseCoverage {
        phase,
        name: name.to_string(),
        status: status.to_string(),
        implementation_mode: implementation_mode.to_string(),
        current_anchor: current_anchor
            .iter()
            .map(|anchor| anchor.to_string())
            .collect(),
        blocked_items: blocked_items.iter().map(|item| item.to_string()).collect(),
        verification: verification
            .iter()
            .map(|command| command.to_string())
            .collect(),
    }
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
