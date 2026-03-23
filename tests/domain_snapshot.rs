use agent_llm_mm::domain::{
    rules::{commitment_gate::gate_decision, snapshot_builder::build_snapshot},
    snapshot::SnapshotRequest,
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
