use std::fs;

use agent_llm_mm::support::product_readiness::{
    ProductReadinessOptions, remote_team_readiness_gate_from_inventory, summarize_product_readiness,
};
use agent_llm_mm::support::remote_team::{
    RemoteTeamCapability, RemoteTeamCapabilityInventory, RemoteTeamCapabilityState,
};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn product_readiness_blocks_simulation_windows_and_missing_release_decision() {
    let temp_dir = tempdir().expect("temp dir");
    write_satisfied_product_smoke(temp_dir.path());
    write_first_run_simulation(temp_dir.path());
    write_support_bundle(temp_dir.path());

    let summary = summarize_product_readiness(ProductReadinessOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: "local-alpha-20260524.1-rc.1".to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("product readiness summary should be generated");

    assert!(!summary.ready);
    assert_eq!(summary.overall_status, "blocked");
    assert_gate(
        &summary,
        "real_fresh_machine",
        "blocked",
        "real fresh-machine evidence is false",
    );
    assert_gate(
        &summary,
        "windows_parity",
        "blocked",
        "missing Windows runtime parity evidence",
    );
    assert_gate(
        &summary,
        "release_decision",
        "blocked",
        "missing release decision summary",
    );
    assert_gate(
        &summary,
        "product_wording",
        "satisfied",
        "candidate wording contains no blocked product claims",
    );
    assert_gate(
        &summary,
        "remote_team",
        "blocked",
        "remote/team capability inventory keeps remote and team features blocked",
    );
    assert_gate(
        &summary,
        "security_auth",
        "blocked",
        "blocked security gates",
    );
    assert!(
        summary
            .non_claims
            .contains(&"not Local Alpha certification".to_string())
    );
}

#[test]
fn product_readiness_blocks_overstated_release_candidate_wording() {
    let temp_dir = tempdir().expect("temp dir");
    write_satisfied_product_smoke(temp_dir.path());
    write_first_run_simulation(temp_dir.path());
    write_support_bundle(temp_dir.path());

    let summary = summarize_product_readiness(ProductReadinessOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: "ga-production-ready-remote-team".to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("product readiness summary should be generated");

    assert!(!summary.ready);
    assert_gate(
        &summary,
        "product_wording",
        "blocked",
        "blocked product claims",
    );
}

#[test]
fn product_readiness_rejects_unsafe_release_candidate_names_before_path_lookup() {
    let temp_dir = tempdir().expect("temp dir");
    write_satisfied_product_smoke(temp_dir.path());
    write_first_run_simulation(temp_dir.path());
    write_support_bundle(temp_dir.path());

    let error = summarize_product_readiness(ProductReadinessOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: "../local-alpha".to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect_err("unsafe candidate names should be rejected");

    assert!(
        error
            .to_string()
            .contains("candidate name must contain only letters"),
        "unexpected error: {error}"
    );
}

#[test]
fn product_readiness_script_exposes_candidate_evidence_gate() {
    let script =
        fs::read_to_string("scripts/product-readiness-check.sh").expect("script should exist");
    let mode = fs::metadata("scripts/product-readiness-check.sh")
        .expect("script metadata")
        .permissions();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_ne!(
            mode.mode() & 0o111,
            0,
            "script should be directly executable"
        );
    }

    assert!(script.contains("usage: ./scripts/product-readiness-check.sh <release-candidate>"));
    assert!(script.contains("candidate name must contain only letters"));
    assert!(script.contains("cargo run --quiet --bin product_readiness_check --"));
    assert!(script.contains("--release-candidate"));
    assert!(
        !script.contains(" ssh "),
        "product readiness check must not call ssh"
    );
    assert!(
        !script.contains(" scp "),
        "product readiness check must not call scp"
    );
    assert!(
        !script.contains(" rsync "),
        "product readiness check must not call rsync"
    );
}

#[test]
fn remote_team_readiness_gate_stays_blocked_when_inventory_exposes_write_or_upload_paths() {
    let gate = remote_team_readiness_gate_from_inventory(&RemoteTeamCapabilityInventory {
        local_only: false,
        support_bundle_upload_available: true,
        capabilities: vec![RemoteTeamCapability {
            name: "remote_write_admin",
            state: RemoteTeamCapabilityState::NotImplemented,
            write_capable_route_exposed: true,
            blocker: "test inventory exposed a write route",
        }],
    });

    assert_eq!(gate.name, "remote_team");
    assert_eq!(gate.status, "blocked");
    assert!(
        gate.reason.contains("support bundle upload is available"),
        "reason should preserve upload blocker detail: {}",
        gate.reason
    );
    assert!(
        gate.reason
            .contains("write-capable remote/team routes are exposed"),
        "reason should preserve write-route blocker detail: {}",
        gate.reason
    );
    assert!(
        gate.reason
            .contains("remote/team capability state is not blocked"),
        "reason should preserve unblocked capability detail: {}",
        gate.reason
    );
}

fn assert_gate(
    summary: &agent_llm_mm::support::product_readiness::ProductReadinessSummary,
    name: &str,
    status: &str,
    reason_contains: &str,
) {
    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == name)
        .unwrap_or_else(|| panic!("missing gate {name}; gates={:?}", summary.gates));

    assert_eq!(gate.status, status);
    assert!(
        gate.reason.contains(reason_contains),
        "gate {name} reason should contain {reason_contains:?}; got {:?}",
        gate.reason
    );
}

fn write_satisfied_product_smoke(root: &std::path::Path) {
    let output_dir = root.join("target/reports/self-revision-demo/latest");
    fs::create_dir_all(&output_dir).expect("create product smoke dir");
    for file in [
        "doctor.json",
        "snapshot-before.json",
        "snapshot-after.json",
        "decision-before.json",
        "decision-after.json",
        "timeline.json",
        "sqlite-summary.json",
        "report.md",
    ] {
        fs::write(output_dir.join(file), "evidence").expect("write product smoke evidence");
    }
}

fn write_first_run_simulation(root: &std::path::Path) {
    let output_dir = root.join("target/first-run-bootstrap-smoke/local-alpha-gate");
    fs::create_dir_all(&output_dir).expect("create first-run dir");
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_vec_pretty(&json!({
            "kind": "local_first_run_bootstrap_simulation",
            "local_only": true,
            "fresh_machine_simulation": true,
            "real_fresh_machine_evidence": false,
            "doctor_status": "ok",
            "self_revision_write_path": "run_reflection",
            "daemon_enabled": false,
            "daemon_writes_allowed": false,
            "sqlite_database_exists": true,
            "started_serve": false,
            "ran_product_smoke": false
        }))
        .expect("json"),
    )
    .expect("write first-run summary");
}

fn write_support_bundle(root: &std::path::Path) {
    let output_dir = root.join("target/support-bundles/local-alpha-gate");
    fs::create_dir_all(&output_dir).expect("create support bundle dir");
    for (file, value) in [
        (
            "manifest.json",
            json!({
                "bundle_format": "agent-llm-mm-local-alpha-support-bundle-v1",
                "local_only": true,
                "upload_performed": false
            }),
        ),
        (
            "doctor.json",
            json!({
                "status": "config-shape-ok",
                "self_revision_write_path": "run_reflection",
                "runtime_bootstrap_performed": false
            }),
        ),
        (
            "product-smoke-summary.json",
            json!({
                "required_artifacts": [
                    "doctor.json",
                    "snapshot-before.json",
                    "snapshot-after.json",
                    "decision-before.json",
                    "decision-after.json",
                    "timeline.json",
                    "sqlite-summary.json",
                    "report.md"
                ],
                "self_revision_write_path_expected": "run_reflection"
            }),
        ),
        ("config-shape.json", json!({ "ok": true })),
        ("operation-summaries.json", json!({ "ok": true })),
        ("release-metadata.json", json!({ "ok": true })),
        ("local-log-excerpts.json", json!({ "ok": true })),
    ] {
        fs::write(
            output_dir.join(file),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .expect("write support bundle file");
    }
}
