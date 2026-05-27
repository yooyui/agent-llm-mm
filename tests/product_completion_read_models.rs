use agent_llm_mm::{
    domain::{
        episode_projection::{EpisodeProjectionInput, build_episode_summary_projection},
        evidence_relation::{EvidenceRelationInput, build_evidence_relation_report},
        memory_layer_projection::{MemoryLayerProjectionInput, build_memory_layer_projection},
        snapshot::SelfSnapshot,
    },
    run_doctor,
    support::{
        product_wording::{ClaimGateState, ProductClaimGuardInput, check_product_claims},
        remote_team::{
            RemoteTeamCapabilityState, remote_team_capability_inventory,
            remote_team_security_gate_report,
        },
    },
};
use tempfile::tempdir;

#[test]
fn evidence_relation_report_keeps_selection_inside_trigger_window_with_ranking_metadata() {
    let report = build_evidence_relation_report(EvidenceRelationInput {
        trigger_window_event_ids: vec![
            "evt-5".to_string(),
            "evt-4".to_string(),
            "evt-3".to_string(),
        ],
        selected_evidence_event_ids: vec!["evt-4".to_string()],
        selection_basis: Some("query_intersection".to_string()),
    })
    .expect("selected evidence is inside the trigger window");

    assert_eq!(report.protocol_version, 2);
    assert_eq!(report.trigger_window_size, 3);
    assert_eq!(report.selected_count, 1);
    assert_eq!(
        report.no_widening_policy,
        "selected_subset_of_trigger_window"
    );
    assert_eq!(report.relations.len(), 3);
    assert_eq!(report.relations[1].event_id, "evt-4");
    assert_eq!(report.relations[1].window_rank, 2);
    assert!(report.relations[1].selected);
    assert_eq!(
        report.relations[1].selection_basis.as_deref(),
        Some("query_intersection")
    );
    assert!(
        report
            .relations
            .iter()
            .all(|relation| relation.event_id != "evt-outside")
    );
}

#[test]
fn evidence_relation_report_rejects_selected_evidence_outside_trigger_window() {
    let error = build_evidence_relation_report(EvidenceRelationInput {
        trigger_window_event_ids: vec!["evt-2".to_string(), "evt-1".to_string()],
        selected_evidence_event_ids: vec!["evt-outside".to_string()],
        selection_basis: Some("model_proposed_ids".to_string()),
    })
    .expect_err("selected evidence outside the current trigger window must be rejected");

    assert!(error.to_string().contains("evt-outside"));
    assert!(error.to_string().contains("outside the trigger window"));
}

#[test]
fn episode_summary_projection_is_read_only_local_metadata_over_episode_events() {
    let projection = build_episode_summary_projection(EpisodeProjectionInput {
        episode_reference: "episode:task-42".to_string(),
        episode_event_ids: vec![
            "evt-objective".to_string(),
            "evt-action".to_string(),
            "evt-outcome".to_string(),
        ],
        objective: Some("stabilize local evidence gate".to_string()),
        outcome: Some("blocked remote claims until auth gates exist".to_string()),
        linked_evidence_ids: vec!["evt-action".to_string(), "evt-outcome".to_string()],
    })
    .expect("episode event projection should build");

    assert_eq!(projection.episode_reference, "episode:task-42");
    assert_eq!(
        projection.objective.as_deref(),
        Some("stabilize local evidence gate")
    );
    assert_eq!(
        projection.linked_evidence_ids,
        ["evt-action", "evt-outcome"]
    );
    assert_eq!(projection.event_count, 3);
    assert!(!projection.writes_performed);
    assert_eq!(projection.durable_self_model_write_path, "run_reflection");
    assert_eq!(
        projection.identity_or_commitment_updates,
        Vec::<String>::new()
    );
}

#[test]
fn remote_team_inventory_keeps_every_remote_or_team_capability_blocked_without_gates() {
    let inventory = remote_team_capability_inventory();

    assert!(inventory.local_only);
    assert!(!inventory.support_bundle_upload_available);
    assert!(
        inventory
            .capabilities
            .iter()
            .filter(|capability| capability.name.starts_with("remote_")
                || capability.name.starts_with("team_"))
            .all(|capability| capability.state == RemoteTeamCapabilityState::Blocked)
    );
    assert!(
        inventory
            .capabilities
            .iter()
            .all(|capability| !capability.write_capable_route_exposed)
    );
}

#[test]
fn remote_team_security_gates_block_remote_writes_until_all_prerequisites_exist() {
    let report = remote_team_security_gate_report();

    assert!(!report.remote_writes_allowed);
    assert!(report.blocked_gate_names().contains(&"auth"));
    assert!(report.blocked_gate_names().contains(&"authorization"));
    assert!(report.blocked_gate_names().contains(&"audit"));
    assert!(report.blocked_gate_names().contains(&"rate_limit"));
    assert!(report.blocked_gate_names().contains(&"tenant_isolation"));
    assert!(report.blocked_gate_names().contains(&"rollback"));
}

#[tokio::test]
async fn doctor_exposes_remote_team_inventory_and_security_gates_as_machine_readable_json() {
    let temp_dir = tempdir().expect("temp dir");
    let report = run_doctor(agent_llm_mm::support::config::AppConfig {
        database_url: format!(
            "sqlite://{}",
            temp_dir
                .path()
                .join("doctor-remote-team.sqlite")
                .to_string_lossy()
        ),
        ..Default::default()
    })
    .await
    .expect("doctor should pass");

    assert!(report.remote_team_capability_inventory.local_only);
    assert!(!report.remote_team_security_gates.remote_writes_allowed);
    assert!(
        report
            .remote_team_capability_inventory
            .capabilities
            .iter()
            .all(|capability| !capability.write_capable_route_exposed)
    );

    let serialized = serde_json::to_value(&report).expect("doctor JSON");
    assert_eq!(
        serialized["remote_team_capability_inventory"]["support_bundle_upload_available"],
        false
    );
    assert_eq!(
        serialized["remote_team_security_gates"]["remote_writes_allowed"],
        false
    );
}

#[tokio::test]
async fn doctor_exposes_read_only_system_layer_report_with_architecture_blockers() {
    let temp_dir = tempdir().expect("temp dir");
    let report = run_doctor(agent_llm_mm::support::config::AppConfig {
        database_url: format!(
            "sqlite://{}",
            temp_dir
                .path()
                .join("doctor-system-layer.sqlite")
                .to_string_lossy()
        ),
        ..Default::default()
    })
    .await
    .expect("doctor should pass");

    let system_layer_report = &report.system_layer_report;
    assert!(system_layer_report.read_only);
    assert!(!system_layer_report.writes_performed);

    let layer_names: Vec<&str> = system_layer_report
        .layers
        .iter()
        .map(|layer| layer.name.as_str())
        .collect();
    assert_eq!(
        layer_names,
        [
            "substrate",
            "signal",
            "memory",
            "policy",
            "control_loop",
            "actuator",
            "interface",
            "release_boundary",
        ]
    );

    let allowed_statuses = [
        "implemented",
        "partial",
        "simulation-only",
        "planning-gate",
        "not-implemented",
    ];
    assert!(
        system_layer_report
            .layers
            .iter()
            .all(|layer| allowed_statuses.contains(&layer.status.as_str()))
    );

    let blockers = system_layer_report.blockers.join("\n").to_lowercase();
    for expected_blocker in [
        "local alpha",
        "windows parity",
        "fresh-machine",
        "remote/team",
        "daemon writes",
        "security/auth",
        "memory layering",
    ] {
        assert!(
            blockers.contains(expected_blocker),
            "missing blocker: {expected_blocker}"
        );
    }

    let physics_principles: Vec<&str> = system_layer_report
        .physics_principles
        .iter()
        .map(|principle| principle.principle.as_str())
        .collect();
    assert_eq!(
        physics_principles,
        [
            "causality",
            "conservation",
            "arrow_of_time",
            "locality",
            "feedback_control",
            "entropy_increase",
            "energy_budget",
            "boundary_conditions",
        ]
    );
    assert!(
        system_layer_report
            .physics_principles
            .iter()
            .all(|principle| principle.report_only && !principle.grants_capability)
    );

    let dependency_rule_names: Vec<&str> = system_layer_report
        .dependency_rules
        .iter()
        .map(|rule| rule.name.as_str())
        .collect();
    assert_eq!(
        dependency_rule_names,
        [
            "actuator_has_no_dashboard_or_release_dependency",
            "write_capable_interface_requires_run_reflection_or_adr",
            "observe_only_daemon_must_not_call_actuator",
            "release_boundary_cannot_generate_external_evidence",
            "memory_writes_require_migration_and_lifecycle_gates",
        ]
    );
    assert!(
        system_layer_report
            .dependency_rules
            .iter()
            .all(|rule| rule.enforced_as == "read-only-boundary")
    );

    let phase_numbers: Vec<u8> = system_layer_report
        .phase_coverage
        .iter()
        .map(|phase| phase.phase)
        .collect();
    assert_eq!(phase_numbers, [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    let phase_coverage =
        serde_json::to_string(&system_layer_report.phase_coverage).expect("phase coverage json");
    for expected_boundary in [
        "real fresh-machine evidence",
        "Windows runtime parity",
        "human release decision",
        "provider adapter expansion",
        "daemon-triggered writes",
        "remote write admin",
        "tenant isolation",
        "installer",
        "Beta/GA claims",
    ] {
        assert!(
            phase_coverage.contains(expected_boundary),
            "missing phase boundary: {expected_boundary}"
        );
    }

    let non_claims = system_layer_report.non_claims.join("\n").to_lowercase();
    for non_claim in [
        "not a physics solver",
        "not a constraint optimizer",
        "not scientific validation evidence",
        "not complete multi-layer cognition",
        "not a remote/team product",
    ] {
        assert!(
            non_claims.contains(non_claim),
            "missing non-claim: {non_claim}"
        );
    }

    let actuator_layer = system_layer_report
        .layers
        .iter()
        .find(|layer| layer.name == "actuator")
        .expect("actuator layer");
    assert!(
        actuator_layer
            .anchors
            .contains(&"run_reflection".to_string())
    );
    assert!(!actuator_layer.writes_allowed);

    let serialized = serde_json::to_value(&report).expect("doctor JSON");
    assert!(serialized.get("system_layer_report").is_some());
    assert_eq!(
        serialized["system_layer_report"]["layers"][5]["writes_allowed"],
        false
    );
    let serialized_text = serde_json::to_string(&report).expect("doctor JSON text");
    assert!(serialized_text.contains("\"system_layer_report\""));
    assert!(serialized_text.contains("\"physics_principles\""));
    assert!(serialized_text.contains("\"dependency_rules\""));
    assert!(serialized_text.contains("\"phase_coverage\""));
    assert!(!serialized_text.contains("\"remote_writes_allowed\":true"));
    assert!(!serialized_text.contains("\"daemon_writes_allowed\":true"));
}

#[test]
fn memory_layer_projection_is_read_only_and_keeps_self_model_durable_writes_blocked() {
    let projection = build_memory_layer_projection(MemoryLayerProjectionInput {
        snapshot: SelfSnapshot {
            identity: vec!["identity:self=technical-demo".to_string()],
            commitments: vec!["forbid:write_identity_core_directly".to_string()],
            claims: vec!["claim:self.role=technical-demo".to_string()],
            evidence: vec!["event:evt-1".to_string()],
            episodes: vec!["episode:task-1".to_string()],
        },
        episode_projection_count: 1,
    });

    assert!(projection.read_only);
    assert!(!projection.writes_performed);
    assert_eq!(projection.durable_self_model_write_path, "run_reflection");
    assert_eq!(
        projection.layer_status("working").as_deref(),
        Some("partial")
    );
    assert_eq!(
        projection.layer_status("episodic").as_deref(),
        Some("partial")
    );
    assert_eq!(
        projection.layer_status("semantic").as_deref(),
        Some("partial")
    );
    assert_eq!(
        projection.layer_status("procedural").as_deref(),
        Some("not_implemented")
    );
    assert_eq!(
        projection.layer_status("self_model").as_deref(),
        Some("partial")
    );
}

#[test]
fn product_wording_guard_blocks_overstated_claims_without_matching_gates() {
    let report = check_product_claims(ProductClaimGuardInput {
        text: "Agent LLM MM is GA, production-ready, supports remote team service, and has complete self-governance.".to_string(),
        gate_state: ClaimGateState::default(),
    });

    assert!(!report.allowed);
    assert_eq!(report.violations.len(), 4);
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.claim == "ga")
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.claim == "production_ready")
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.claim == "remote_team_service")
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.claim == "complete_self_governance")
    );
}
