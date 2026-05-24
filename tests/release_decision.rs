use std::fs;

use agent_llm_mm::support::release_decision::{
    ReleaseDecisionOptions, ReleaseDecisionRequest, write_release_decision,
};
use tempfile::tempdir;

#[test]
fn release_decision_rejects_approval_when_local_alpha_evidence_is_in_progress() {
    let temp_dir = tempdir().expect("temp dir");

    let error = write_release_decision(ReleaseDecisionOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: "local-alpha-20260524.1-rc.1".to_string(),
        request: ReleaseDecisionRequest {
            decision: "approved".to_string(),
            human_reviewer: Some("maintainer@example.test".to_string()),
            rollback_note: Some("Revert the source-only candidate tag.".to_string()),
        },
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect_err("approval must be rejected while evidence is still in progress");

    assert!(
        error.to_string().contains(
            "cannot approve release decision while Local Alpha evidence status is in_progress"
        ),
        "error should explain the evidence status blocker: {error}"
    );
}

#[test]
fn release_decision_writes_blocked_template_with_open_gates() {
    let temp_dir = tempdir().expect("temp dir");
    let output_json = temp_dir.path().join("release-decision.json");
    let output_md = temp_dir.path().join("release-decision.md");

    let artifact = write_release_decision(ReleaseDecisionOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: "local-alpha-20260524.1-rc.1".to_string(),
        request: ReleaseDecisionRequest {
            decision: "blocked".to_string(),
            human_reviewer: None,
            rollback_note: None,
        },
        output_json_path: Some(output_json.clone()),
        output_markdown_path: Some(output_md.clone()),
    })
    .expect("blocked template should be writable even when gates are open");

    assert_eq!(artifact.kind, "release_decision");
    assert_eq!(artifact.release_candidate, "local-alpha-20260524.1-rc.1");
    assert_eq!(artifact.decision, "blocked");
    assert!(!artifact.approved);
    assert_eq!(artifact.evidence_status, "in_progress");
    assert!(
        artifact
            .open_gates
            .iter()
            .any(|gate| gate.name == "product_smoke"),
        "blocked decision should carry open gate context"
    );
    assert!(
        artifact
            .non_claims
            .contains(&"not release approval".to_string())
    );

    let written = fs::read_to_string(output_json).expect("json output");
    assert!(written.contains(r#""decision": "blocked""#));
    assert!(
        fs::read_to_string(output_md)
            .expect("markdown output")
            .contains("Release Decision")
    );
}

#[test]
fn release_decision_script_is_local_source_only_and_writes_candidate_artifacts() {
    let script =
        fs::read_to_string("scripts/release-decision-local.sh").expect("script should exist");

    assert!(script.contains("usage: ./scripts/release-decision-local.sh <release-candidate>"));
    assert!(script.contains("target/reports/releases"));
    assert!(script.contains("release-decision.json"));
    assert!(script.contains("release-decision.md"));
    assert!(script.contains("cargo run --quiet --bin release_decision_local --"));
    assert!(
        !script.contains(" ssh "),
        "release decision script must not call ssh"
    );
    assert!(
        !script.contains(" scp "),
        "release decision script must not call scp"
    );
    assert!(
        !script.contains(" rsync "),
        "release decision script must not call rsync"
    );
}
