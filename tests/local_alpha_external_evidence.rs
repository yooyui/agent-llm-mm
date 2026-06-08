use std::fs;

use agent_llm_mm::support::local_alpha_evidence::{
    LocalAlphaEvidenceOptions, summarize_local_alpha_evidence,
};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn real_fresh_machine_gate_requires_external_evidence_metadata() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("first-run-bootstrap/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create first-run evidence dir");
    fs::write(
        &summary_path,
        json!({
            "doctor_status": "ok",
            "fresh_machine_simulation": false,
            "real_fresh_machine_evidence": true,
            "local_only": true,
            "self_revision_write_path": "run_reflection",
            "daemon_enabled": false,
            "daemon_writes_allowed": false,
            "started_serve": false,
            "ran_product_smoke": false,
            "sqlite_database_exists": true
        })
        .to_string(),
    )
    .expect("write incomplete real fresh-machine evidence");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "first_run_bootstrap")
        .expect("first-run gate");
    assert_eq!(gate.status, "open");
    assert!(
        gate.reason
            .contains("real fresh-machine evidence metadata is incomplete"),
        "reason should explain missing real evidence metadata: {}",
        gate.reason
    );
}

#[test]
fn real_fresh_machine_gate_rejects_mixed_simulation_and_real_evidence_markers() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("first-run-bootstrap/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create first-run evidence dir");
    fs::write(
        &summary_path,
        json!({
            "kind": "local_first_run_bootstrap_simulation",
            "evidence_kind": "real_fresh_machine_first_run",
            "captured_at": "2026-06-08T00:00:00Z",
            "source_checkout": "clean-clone-or-unpacked-archive",
            "command_evidence": [
                {
                    "name": "bootstrap-local",
                    "command": "./scripts/agent-llm-mm.sh bootstrap-local",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "doctor",
                    "command": "./scripts/agent-llm-mm.sh doctor",
                    "status": "passed",
                    "exit_code": 0
                }
            ],
            "doctor_status": "ok",
            "fresh_machine_simulation": true,
            "real_fresh_machine_evidence": true,
            "local_only": true,
            "self_revision_write_path": "run_reflection",
            "daemon_enabled": false,
            "daemon_writes_allowed": false,
            "started_serve": false,
            "ran_product_smoke": false,
            "sqlite_database_exists": true
        })
        .to_string(),
    )
    .expect("write mixed first-run evidence");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "first_run_bootstrap")
        .expect("first-run gate");
    assert_eq!(gate.status, "open");
    assert!(
        gate.reason
            .contains("real fresh-machine evidence conflicts with simulation metadata"),
        "reason should reject mixed simulation/real evidence: {}",
        gate.reason
    );
}

#[test]
fn real_fresh_machine_gate_requires_explicit_success_exit_codes() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("first-run-bootstrap/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create first-run evidence dir");
    fs::write(
        &summary_path,
        json!({
            "kind": "real_first_run_bootstrap_evidence",
            "evidence_kind": "real_fresh_machine_first_run",
            "captured_at": "2026-06-08T00:00:00Z",
            "source_checkout": "clean-clone-or-unpacked-archive",
            "command_evidence": [
                {
                    "name": "bootstrap-local",
                    "command": "./scripts/agent-llm-mm.sh bootstrap-local",
                    "status": "passed"
                },
                {
                    "name": "doctor",
                    "command": "./scripts/agent-llm-mm.sh doctor",
                    "status": "passed",
                    "exit_code": 0
                }
            ],
            "doctor_status": "ok",
            "fresh_machine_simulation": false,
            "real_fresh_machine_evidence": true,
            "local_only": true,
            "self_revision_write_path": "run_reflection",
            "daemon_enabled": false,
            "daemon_writes_allowed": false,
            "started_serve": false,
            "ran_product_smoke": false,
            "sqlite_database_exists": true
        })
        .to_string(),
    )
    .expect("write first-run evidence without explicit exit code");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "first_run_bootstrap")
        .expect("first-run gate");
    assert_eq!(gate.status, "open");
    assert!(
        gate.reason
            .contains("real fresh-machine evidence metadata is incomplete"),
        "reason should reject missing command exit code: {}",
        gate.reason
    );
}

#[test]
fn real_fresh_machine_gate_accepts_external_command_evidence_metadata() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("first-run-bootstrap/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create first-run evidence dir");
    fs::write(
        &summary_path,
        json!({
            "kind": "real_first_run_bootstrap_evidence",
            "evidence_kind": "real_fresh_machine_first_run",
            "captured_at": "2026-06-08T00:00:00Z",
            "source_checkout": "clean-clone-or-unpacked-archive",
            "command_evidence": [
                {
                    "name": "bootstrap-local",
                    "command": "./scripts/agent-llm-mm.sh bootstrap-local",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "doctor",
                    "command": "./scripts/agent-llm-mm.sh doctor",
                    "status": "passed",
                    "exit_code": 0
                }
            ],
            "doctor_status": "ok",
            "fresh_machine_simulation": false,
            "real_fresh_machine_evidence": true,
            "local_only": true,
            "self_revision_write_path": "run_reflection",
            "daemon_enabled": false,
            "daemon_writes_allowed": false,
            "started_serve": false,
            "ran_product_smoke": false,
            "sqlite_database_exists": true
        })
        .to_string(),
    )
    .expect("write complete real fresh-machine evidence");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "first_run_bootstrap")
        .expect("first-run gate");
    assert_eq!(gate.status, "satisfied");
    assert!(
        gate.reason
            .contains("real fresh-machine first-run evidence is present and local-only"),
        "reason should explain accepted real evidence: {}",
        gate.reason
    );
}

#[test]
fn windows_parity_gate_requires_runtime_command_evidence() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("windows-parity/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create windows parity evidence dir");
    fs::write(
        &summary_path,
        json!({
            "status": "verified",
            "runner": "github-actions-windows",
            "platform": "windows-latest",
            "runtime_parity": true
        })
        .to_string(),
    )
    .expect("write incomplete windows parity evidence");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "windows_parity")
        .expect("windows parity gate");
    assert_eq!(gate.status, "not_verified");
    assert!(
        gate.reason
            .contains("Windows runtime command evidence is incomplete"),
        "reason should explain missing Windows command evidence: {}",
        gate.reason
    );
}

#[test]
fn windows_parity_gate_requires_explicit_success_exit_codes() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("windows-parity/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create windows parity evidence dir");
    fs::write(
        &summary_path,
        json!({
            "evidence_kind": "windows_runtime_parity",
            "captured_at": "2026-06-08T00:00:00Z",
            "status": "verified",
            "runner": "github-actions-windows",
            "platform": "windows-latest",
            "runtime_parity": true,
            "command_evidence": [
                {
                    "name": "bootstrap-local",
                    "command": "pwsh -File scripts/agent-llm-mm.ps1 bootstrap-local",
                    "status": "passed"
                },
                {
                    "name": "doctor",
                    "command": "pwsh -File scripts/agent-llm-mm.ps1 doctor",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "product-smoke",
                    "command": "pwsh -File scripts/product-smoke-local.ps1",
                    "status": "passed",
                    "exit_code": 0
                }
            ]
        })
        .to_string(),
    )
    .expect("write windows parity evidence without explicit exit code");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "windows_parity")
        .expect("windows parity gate");
    assert_eq!(gate.status, "not_verified");
    assert!(
        gate.reason
            .contains("Windows runtime command evidence is incomplete"),
        "reason should reject missing command exit code: {}",
        gate.reason
    );
}

#[test]
fn windows_parity_gate_accepts_runtime_command_evidence_metadata() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("windows-parity/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create windows parity evidence dir");
    fs::write(
        &summary_path,
        json!({
            "evidence_kind": "windows_runtime_parity",
            "captured_at": "2026-06-08T00:00:00Z",
            "status": "verified",
            "runner": "github-actions-windows",
            "platform": "windows-latest",
            "runtime_parity": true,
            "command_evidence": [
                {
                    "name": "bootstrap-local",
                    "command": "pwsh -File scripts/agent-llm-mm.ps1 bootstrap-local",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "doctor",
                    "command": "pwsh -File scripts/agent-llm-mm.ps1 doctor",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "product-smoke",
                    "command": "pwsh -File scripts/product-smoke-local.ps1",
                    "status": "passed",
                    "exit_code": 0
                }
            ]
        })
        .to_string(),
    )
    .expect("write complete windows parity evidence");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "windows_parity")
        .expect("windows parity gate");
    assert_eq!(gate.status, "satisfied");
    assert!(
        gate.reason
            .contains("Windows runtime parity evidence is verified"),
        "reason should explain accepted Windows evidence: {}",
        gate.reason
    );
}

#[test]
fn windows_parity_gate_accepts_generic_runner_when_platform_is_windows() {
    let temp_dir = tempdir().expect("temp dir");
    let summary_path = temp_dir.path().join("windows-parity/summary.json");
    fs::create_dir_all(summary_path.parent().expect("summary parent"))
        .expect("create windows parity evidence dir");
    fs::write(
        &summary_path,
        json!({
            "evidence_kind": "windows_runtime_parity",
            "captured_at": "2026-06-08T00:00:00Z",
            "status": "verified",
            "runner": "github-actions",
            "platform": "windows-latest",
            "runtime_parity": true,
            "command_evidence": [
                {
                    "name": "bootstrap-local",
                    "command": "pwsh -File scripts/agent-llm-mm.ps1 bootstrap-local",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "doctor",
                    "command": "pwsh -File scripts/agent-llm-mm.ps1 doctor",
                    "status": "passed",
                    "exit_code": 0
                },
                {
                    "name": "product-smoke",
                    "command": "pwsh -File scripts/product-smoke-local.ps1",
                    "status": "passed",
                    "exit_code": 0
                }
            ]
        })
        .to_string(),
    )
    .expect("write complete windows parity evidence");

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("summarize local alpha evidence");

    let gate = summary
        .gates
        .iter()
        .find(|gate| gate.name == "windows_parity")
        .expect("windows parity gate");
    assert_eq!(gate.status, "satisfied");
    assert!(
        gate.reason
            .contains("Windows runtime parity evidence is verified"),
        "reason should accept Windows platform independent of runner: {}",
        gate.reason
    );
}
