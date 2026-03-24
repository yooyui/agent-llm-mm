use agent_llm_mm::domain::{
    DomainError,
    rules::{commitment_gate::gate_decision, snapshot_builder::build_snapshot},
    snapshot::{SnapshotBudget, SnapshotRequest},
};

#[test]
fn snapshot_summary_includes_supporting_event_reference() {
    let snapshot = build_snapshot(SnapshotRequest::fixture_minimal()).unwrap();
    assert!(!snapshot.evidence.is_empty());
}

#[test]
fn hard_commitment_blocks_conflicting_action() {
    let result = gate_decision(
        "write_identity_core_directly",
        &SnapshotRequest::fixture_minimal().commitments,
    );
    assert!(result.blocked);
}

#[test]
fn budget_of_zero_still_recalls_one_evidence_item() {
    let mut request = SnapshotRequest::fixture_minimal();
    request.budget = SnapshotBudget::new(0);

    let snapshot = build_snapshot(request).unwrap();

    assert_eq!(snapshot.evidence.len(), 1);
}

#[test]
fn snapshot_without_evidence_reuses_insufficient_evidence_error() {
    let mut request = SnapshotRequest::fixture_minimal();
    request.evidence.clear();

    let error = build_snapshot(request).unwrap_err();

    assert_eq!(error, DomainError::InsufficientEvidence);
}

#[test]
fn snapshot_budget_truncates_evidence_to_limit() {
    let mut request = SnapshotRequest::fixture_minimal();
    request.evidence = vec![
        "event:evt-1".to_string(),
        "event:evt-2".to_string(),
        "event:evt-3".to_string(),
    ];
    request.budget = SnapshotBudget::new(2);

    let snapshot = build_snapshot(request).unwrap();

    assert_eq!(
        snapshot.evidence,
        vec!["event:evt-1".to_string(), "event:evt-2".to_string()]
    );
}

#[test]
fn unrelated_action_is_not_blocked_by_commitment_gate() {
    let result = gate_decision(
        "read_identity_core",
        &SnapshotRequest::fixture_minimal().commitments,
    );

    assert!(!result.blocked);
}
