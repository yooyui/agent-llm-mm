use agent_llm_mm::domain::{
    claim::DomainError,
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
fn snapshot_budget_of_zero_does_not_select_evidence() {
    let mut request = SnapshotRequest::fixture_minimal();
    request.budget = SnapshotBudget::new(0);

    let error = build_snapshot(request).unwrap_err();

    assert_eq!(error, DomainError::InsufficientEvidence);
}

#[test]
fn snapshot_without_evidence_reuses_insufficient_evidence_error() {
    let mut request = SnapshotRequest::fixture_minimal();
    request.evidence.clear();

    let error = build_snapshot(request).unwrap_err();

    assert_eq!(error, DomainError::InsufficientEvidence);
}
