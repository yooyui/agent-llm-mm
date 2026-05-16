use std::fs;

use agent_llm_mm::support::local_alpha_evidence::{
    LocalAlphaEvidenceOptions, summarize_local_alpha_evidence,
};
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn missing_required_evidence_artifacts_keep_gate_open() {
    let temp_dir = tempdir().expect("temp dir");
    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated from missing evidence");

    assert_eq!(summary.overall_status, "in_progress");
    assert_gate(
        &summary_json(&summary),
        "product_smoke",
        "open",
        "missing or empty product smoke latest evidence",
    );
    assert_gate(
        &summary_json(&summary),
        "first_run_bootstrap",
        "open",
        "missing first-run bootstrap summary",
    );
}

#[test]
fn first_run_simulation_without_real_fresh_machine_evidence_cannot_complete_local_alpha() {
    let temp_dir = tempdir().expect("temp dir");
    write_first_run_summary(temp_dir.path(), false);
    write_product_smoke_latest(temp_dir.path());
    write_windows_parity(temp_dir.path(), "verified");
    write_support_bundle(temp_dir.path(), &allowed_support_bundle_files());

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");
    let summary_json = summary_json(&summary);

    assert_ne!(summary.overall_status, "complete");
    assert_eq!(summary.overall_status, "in_progress");
    assert_gate(
        &summary_json,
        "first_run_bootstrap",
        "open",
        "real fresh-machine evidence is false",
    );
    assert!(
        summary.markdown.contains("Local Alpha is not complete"),
        "markdown must use conservative wording"
    );
}

#[test]
fn missing_windows_parity_is_not_verified() {
    let temp_dir = tempdir().expect("temp dir");
    write_first_run_summary(temp_dir.path(), true);
    write_product_smoke_latest(temp_dir.path());
    write_support_bundle(temp_dir.path(), &allowed_support_bundle_files());

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");
    let summary_json = summary_json(&summary);

    assert_gate(
        &summary_json,
        "windows_parity",
        "not_verified",
        "missing Windows runtime parity evidence",
    );
    assert_ne!(summary.overall_status, "complete");
}

#[test]
fn incomplete_first_run_summary_does_not_satisfy_read_only_boundary_gate() {
    let temp_dir = tempdir().expect("temp dir");
    let output_dir = temp_dir.path().join("first-run-bootstrap");
    fs::create_dir_all(&output_dir).expect("create first-run dir");
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "kind": "local_first_run_bootstrap_simulation"
        }))
        .expect("json"),
    )
    .expect("write incomplete first-run summary");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_gate(
        &summary_json(&summary),
        "local_read_only_boundary",
        "not_verified",
        "first-run boundary evidence is incomplete",
    );
}

#[test]
fn boundary_gate_requires_explicit_local_only_and_daemon_disabled_evidence() {
    let temp_dir = tempdir().expect("temp dir");
    let output_dir = temp_dir.path().join("first-run-bootstrap");
    fs::create_dir_all(&output_dir).expect("create first-run dir");
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "kind": "local_first_run_bootstrap_simulation",
            "started_serve": false,
            "ran_product_smoke": false,
            "self_revision_write_path": "run_reflection",
            "daemon_writes_allowed": false
        }))
        .expect("json"),
    )
    .expect("write incomplete first-run summary");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_gate(
        &summary_json(&summary),
        "local_read_only_boundary",
        "not_verified",
        "first-run boundary evidence is incomplete",
    );
}

#[test]
fn windows_parity_without_windows_runner_evidence_is_not_verified() {
    let temp_dir = tempdir().expect("temp dir");
    let output_dir = temp_dir.path().join("windows-parity");
    fs::create_dir_all(&output_dir).expect("create windows dir");
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "verified",
            "runtime_parity": true
        }))
        .expect("json"),
    )
    .expect("write windows parity");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_gate(
        &summary_json(&summary),
        "windows_parity",
        "not_verified",
        "missing Windows runner or platform evidence",
    );
}

#[test]
fn empty_product_smoke_artifact_keeps_gate_open() {
    let temp_dir = tempdir().expect("temp dir");
    write_product_smoke_latest(temp_dir.path());
    let empty_artifact = temp_dir
        .path()
        .join("target/reports/self-revision-demo/latest/report.md");
    fs::write(empty_artifact, "").expect("empty report artifact");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_gate(
        &summary_json(&summary),
        "product_smoke",
        "open",
        "missing or empty product smoke latest evidence",
    );
}

#[test]
fn support_bundle_with_only_allowed_diagnostic_files_is_satisfied() {
    let temp_dir = tempdir().expect("temp dir");
    write_support_bundle(temp_dir.path(), &allowed_support_bundle_files());

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_gate(
        &summary_json(&summary),
        "support_bundle",
        "satisfied",
        "allowed local diagnostic files present",
    );
}

#[test]
fn support_bundle_placeholder_files_do_not_satisfy_gate() {
    let temp_dir = tempdir().expect("temp dir");
    write_placeholder_support_bundle(temp_dir.path(), &allowed_support_bundle_files());

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_gate(
        &summary_json(&summary),
        "support_bundle",
        "open",
        "support bundle manifest is not valid local alpha evidence",
    );
}

#[test]
fn satisfied_evidence_keeps_overall_review_ready_not_automatically_complete() {
    let temp_dir = tempdir().expect("temp dir");
    write_first_run_summary(temp_dir.path(), true);
    write_product_smoke_latest(temp_dir.path());
    write_windows_parity(temp_dir.path(), "verified");
    write_support_bundle(temp_dir.path(), &allowed_support_bundle_files());

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");

    assert_eq!(summary.overall_status, "ready_for_human_review");
    assert!(
        summary
            .markdown
            .contains("human release decision is still required"),
        "markdown must not imply automatic Local Alpha certification"
    );
}

#[test]
fn summary_reads_documented_target_evidence_paths_by_default() {
    let temp_dir = tempdir().expect("temp dir");
    write_first_run_summary_at(
        &temp_dir
            .path()
            .join("target/first-run-bootstrap-smoke/local-alpha-gate"),
        true,
    );
    write_product_smoke_latest(temp_dir.path());
    write_windows_parity_at(
        &temp_dir
            .path()
            .join("target/windows-parity/local-alpha-gate"),
        "verified",
    );
    write_support_bundle_at(
        &temp_dir
            .path()
            .join("target/support-bundles/local-alpha-gate"),
        &allowed_support_bundle_files(),
    );

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summary should be generated");
    let summary_json = summary_json(&summary);

    assert_gate(
        &summary_json,
        "first_run_bootstrap",
        "satisfied",
        "real fresh-machine first-run evidence is present and local-only",
    );
    assert_gate(
        &summary_json,
        "support_bundle",
        "satisfied",
        "allowed local diagnostic files present",
    );
    assert_gate(
        &summary_json,
        "windows_parity",
        "satisfied",
        "Windows runtime parity evidence is verified",
    );
}

#[test]
fn summary_capability_is_read_only_and_does_not_call_runtime_write_or_remote_paths() {
    let script = fs::read_to_string("scripts/local-alpha-evidence-summary.sh")
        .expect("summary wrapper should be readable");
    let binary = fs::read_to_string("src/bin/local_alpha_evidence_summary.rs")
        .expect("summary binary should be readable");
    let module = fs::read_to_string("src/support/local_alpha_evidence.rs")
        .expect("summary module should be readable");
    let combined = format!("{script}\n{binary}\n{module}");

    assert!(
        !combined.contains("agent-llm-mm.sh serve") && !combined.contains(" serve "),
        "summary capability must not start serve"
    );
    assert!(
        !combined.contains("product-smoke-local.sh"),
        "summary capability must not execute product smoke"
    );
    assert!(
        !combined.contains("run-self-revision-demo.sh"),
        "summary capability must not execute demo smoke"
    );
    assert!(
        !combined.contains(" ssh "),
        "summary capability must not call ssh"
    );
    assert!(
        !combined.contains(" scp "),
        "summary capability must not call scp"
    );
    assert!(
        !combined.contains(" rsync "),
        "summary capability must not call rsync"
    );
    assert!(
        !combined.contains("daemon start"),
        "summary capability must not start daemon behavior"
    );
    assert!(
        !combined.contains("run_reflection(")
            && !combined.contains("AppCommand::Serve")
            && !combined.contains("run_command("),
        "summary capability must not invoke runtime serve/reflection write paths"
    );
}

#[test]
fn binary_rejects_missing_option_values_before_generating_summary() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_local_alpha_evidence_summary"))
        .arg("--output-json")
        .output()
        .expect("summary binary should run");

    assert!(
        !output.status.success(),
        "missing option value should fail instead of silently falling back"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing value for --output-json"),
        "stderr should name the missing option value"
    );
}

fn summary_json(
    summary: &agent_llm_mm::support::local_alpha_evidence::LocalAlphaEvidenceSummary,
) -> Value {
    serde_json::to_value(summary).expect("summary serializes")
}

fn assert_gate(summary: &Value, name: &str, status: &str, reason_contains: &str) {
    let gates = summary["gates"].as_array().expect("gates array");
    let gate = gates
        .iter()
        .find(|gate| gate["name"] == name)
        .unwrap_or_else(|| panic!("missing gate {name}; gates={gates:?}"));

    assert_eq!(gate["status"], status);
    let reason = gate["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains(reason_contains),
        "gate {name} reason should contain {reason_contains:?}; got {reason:?}"
    );
}

fn write_first_run_summary(root: &std::path::Path, real_fresh_machine_evidence: bool) {
    let output_dir = root.join("first-run-bootstrap");
    write_first_run_summary_at(&output_dir, real_fresh_machine_evidence);
}

fn write_first_run_summary_at(output_dir: &std::path::Path, real_fresh_machine_evidence: bool) {
    fs::create_dir_all(output_dir).expect("create first-run dir");
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "kind": "local_first_run_bootstrap_simulation",
            "local_only": true,
            "fresh_machine_simulation": true,
            "real_fresh_machine_evidence": real_fresh_machine_evidence,
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

fn write_product_smoke_latest(root: &std::path::Path) {
    let output_dir = root
        .join("target")
        .join("reports")
        .join("self-revision-demo")
        .join("latest");
    fs::create_dir_all(&output_dir).expect("create latest dir");
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

fn write_windows_parity(root: &std::path::Path, status: &str) {
    let output_dir = root.join("windows-parity");
    write_windows_parity_at(&output_dir, status);
}

fn write_windows_parity_at(output_dir: &std::path::Path, status: &str) {
    fs::create_dir_all(output_dir).expect("create windows dir");
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": status,
            "runner": "windows",
            "runtime_parity": true
        }))
        .expect("json"),
    )
    .expect("write windows parity");
}

fn write_support_bundle(root: &std::path::Path, files: &[&str]) {
    let output_dir = root.join("support-bundle");
    write_support_bundle_at(&output_dir, files);
}

fn write_support_bundle_at(output_dir: &std::path::Path, files: &[&str]) {
    fs::create_dir_all(output_dir).expect("create support bundle dir");
    for file in files {
        fs::write(output_dir.join(file), support_bundle_file_json(file))
            .expect("write support file");
    }
}

fn write_placeholder_support_bundle(root: &std::path::Path, files: &[&str]) {
    let output_dir = root.join("support-bundle");
    fs::create_dir_all(&output_dir).expect("create support bundle dir");
    for file in files {
        fs::write(output_dir.join(file), "{}").expect("write placeholder support file");
    }
}

fn allowed_support_bundle_files() -> Vec<&'static str> {
    vec![
        "manifest.json",
        "doctor.json",
        "config-shape.json",
        "operation-summaries.json",
        "release-metadata.json",
        "product-smoke-summary.json",
        "local-log-excerpts.json",
    ]
}

fn support_bundle_file_json(file: &str) -> Vec<u8> {
    let value = match file {
        "manifest.json" => serde_json::json!({
            "bundle_format": "agent-llm-mm-local-alpha-support-bundle-v1",
            "local_only": true,
            "upload_performed": false,
            "files": allowed_support_bundle_files()
        }),
        "doctor.json" => serde_json::json!({
            "status": "config-shape-ok",
            "self_revision_write_path": "run_reflection",
            "runtime_bootstrap_performed": false
        }),
        "product-smoke-summary.json" => serde_json::json!({
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
            "latest_artifacts_present": true,
            "self_revision_write_path_expected": "run_reflection"
        }),
        "operation-summaries.json" => serde_json::json!({
            "limit": 25,
            "available": false,
            "entries": []
        }),
        "local-log-excerpts.json" => serde_json::json!({
            "available": false,
            "read_only": true,
            "excerpts": []
        }),
        "config-shape.json" => serde_json::json!({
            "database_url_shape": "sqlite://<local-path>"
        }),
        "release-metadata.json" => serde_json::json!({
            "generated_at": "2026-05-16T00:00:00Z"
        }),
        other => panic!("unexpected support bundle file {other}"),
    };
    serde_json::to_vec_pretty(&value).expect("support bundle fixture json")
}
