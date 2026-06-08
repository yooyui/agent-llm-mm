use std::{fs, process::Command};

use agent_llm_mm::support::{
    config::{AppConfig, ModelConfig, ModelProviderKind, OpenAiCompatibleConfig},
    provider_certification::{ProviderCertificationOptions, summarize_provider_certification},
    provider_live_certification::{
        ProviderLiveCertificationMode, ProviderLiveCertificationOptions,
        run_provider_live_certification,
    },
};
use serde_json::Value;
use tempfile::tempdir;

const EVIDENCE_FILES: &[(&str, &str)] = &[
    ("live-decision.json", "live_decision"),
    ("live-self-revision.json", "live_self_revision"),
    ("provider-error-handling.json", "provider_error_handling"),
    ("redaction-review.json", "redaction_review"),
];

#[test]
fn stub_evidence_runner_generates_non_live_provider_evidence_without_certifying_preflight() {
    let temp_dir = tempdir().expect("temp dir");
    let config = openrouter_config();

    let report = run_provider_live_certification(ProviderLiveCertificationOptions {
        config: config.clone(),
        evidence_root: temp_dir.path().to_path_buf(),
        mode: ProviderLiveCertificationMode::StubEvidence,
    })
    .expect("stub evidence generation should succeed");

    assert_eq!(report.provider, "openrouter");
    assert_eq!(report.status, "passed");
    assert_eq!(report.generated_evidence.len(), EVIDENCE_FILES.len());

    let evidence_dir = temp_dir
        .path()
        .join("target/reports/provider-certification/openrouter");
    for (file_name, evidence_kind) in EVIDENCE_FILES {
        let path = evidence_dir.join(file_name);
        assert!(path.exists(), "missing generated evidence file {file_name}");
        let bytes = fs::read(&path).expect("read evidence");
        assert!(!bytes.is_empty(), "evidence file must be non-empty JSON");
        let value: Value = serde_json::from_slice(&bytes).expect("evidence JSON");
        assert_eq!(value["provider"], "openrouter");
        assert_eq!(value["status"], "passed");
        assert_eq!(value["evidence_kind"], *evidence_kind);
        assert_eq!(value["local_only"], true);
        assert_eq!(value["mode"], "stub/simulated");
        assert_eq!(
            value["endpoint_shape"],
            "https://openrouter.example.test/<redacted-path>"
        );
        assert_eq!(value["credential_configured"], true);
        assert!(
            value["generated_at"]
                .as_str()
                .is_some_and(|stamp| !stamp.is_empty()),
            "generated_at must be present"
        );
        assert!(
            value["non_claims"]
                .as_array()
                .expect("non_claims array")
                .iter()
                .any(|claim| claim
                    .as_str()
                    .is_some_and(|claim| claim.contains("not real live provider evidence"))),
            "stub evidence must clearly avoid live-provider claims"
        );
    }

    let summary = summarize_provider_certification(ProviderCertificationOptions {
        config,
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("provider certification preflight");

    assert!(!summary.live_certified);
    assert_eq!(summary.live_certification_status, "blocked");
    assert_eq!(summary.missing_live_evidence.len(), EVIDENCE_FILES.len());
    assert!(
        summary
            .live_evidence
            .iter()
            .all(|entry| entry.status == "invalid"),
        "stub evidence must not be accepted as live certification evidence: {:?}",
        summary.live_evidence
    );
}

#[test]
fn stub_evidence_serialization_redacts_secrets_and_url_sensitive_parts() {
    let temp_dir = tempdir().expect("temp dir");
    let config = AppConfig {
        model_provider: ModelProviderKind::OpenAiCompatible,
        model_config: ModelConfig::OpenAiCompatible(OpenAiCompatibleConfig {
            base_url: "https://url-user:url-password@provider.example.test/api/sk-path-secret/v1?token=query-secret".to_string(),
            api_key: "api-secret-key".to_string(),
            model: "provider/test-model".to_string(),
            timeout_ms: 30_000,
        }),
        ..Default::default()
    };

    run_provider_live_certification(ProviderLiveCertificationOptions {
        config,
        evidence_root: temp_dir.path().to_path_buf(),
        mode: ProviderLiveCertificationMode::StubEvidence,
    })
    .expect("stub evidence generation should succeed");

    let evidence_dir = temp_dir
        .path()
        .join("target/reports/provider-certification/openai-compatible");
    let serialized = EVIDENCE_FILES
        .iter()
        .map(|(file_name, _)| fs::read_to_string(evidence_dir.join(file_name)).expect("evidence"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(serialized.contains("https://provider.example.test/<redacted-path>"));
    for secret in [
        "url-user",
        "url-password",
        "sk-path-secret",
        "query-secret",
        "api-secret-key",
        "provider/test-model",
        "/api/",
    ] {
        assert!(
            !serialized.contains(secret),
            "live certification evidence must not serialize secret or provider-native fragment: {secret}"
        );
    }
}

#[test]
fn preflight_remains_blocked_when_live_evidence_has_not_been_generated() {
    let temp_dir = tempdir().expect("temp dir");

    let summary = summarize_provider_certification(ProviderCertificationOptions {
        config: openrouter_config(),
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("provider certification preflight");

    assert!(!summary.live_certified);
    assert_eq!(summary.live_certification_status, "blocked");
    assert_eq!(summary.missing_live_evidence.len(), EVIDENCE_FILES.len());
}

#[test]
fn preflight_accepts_explicit_live_evidence_files_only() {
    let temp_dir = tempdir().expect("temp dir");
    let evidence_dir = temp_dir
        .path()
        .join("target/reports/provider-certification/openrouter");
    fs::create_dir_all(&evidence_dir).expect("create provider evidence dir");
    for (file_name, evidence_kind) in EVIDENCE_FILES {
        fs::write(
            evidence_dir.join(file_name),
            serde_json::to_vec_pretty(&serde_json::json!({
                "provider": "openrouter",
                "status": "passed",
                "evidence_kind": evidence_kind,
                "mode": "live",
                "generated_at": "2026-06-08T00:00:00Z",
                "local_only": false,
                "endpoint_reached": true,
                "redaction_reviewed": true,
                "request_outcome": "passed",
                "command_evidence": [
                    {
                        "name": "provider-live-certification",
                        "command": "scripts/provider-live-certification-run.sh --live",
                        "status": "passed",
                        "exit_code": 0
                    }
                ]
            }))
            .expect("json"),
        )
        .expect("write live evidence");
    }

    let summary = summarize_provider_certification(ProviderCertificationOptions {
        config: openrouter_config(),
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("provider certification preflight");

    assert!(summary.live_certified);
    assert_eq!(summary.live_certification_status, "passed");
    assert!(summary.missing_live_evidence.is_empty());
    assert!(
        summary
            .live_evidence
            .iter()
            .all(|entry| entry.status == "present")
    );
}

#[test]
fn preflight_rejects_thin_self_labeled_live_evidence_files() {
    let temp_dir = tempdir().expect("temp dir");
    let evidence_dir = temp_dir
        .path()
        .join("target/reports/provider-certification/openrouter");
    fs::create_dir_all(&evidence_dir).expect("create provider evidence dir");
    for (file_name, evidence_kind) in EVIDENCE_FILES {
        fs::write(
            evidence_dir.join(file_name),
            serde_json::json!({
                "provider": "openrouter",
                "status": "passed",
                "evidence_kind": evidence_kind,
                "mode": "live",
                "generated_at": "2026-06-08T00:00:00Z",
                "local_only": false
            })
            .to_string(),
        )
        .expect("write thin live evidence");
    }

    let summary = summarize_provider_certification(ProviderCertificationOptions {
        config: openrouter_config(),
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("provider certification preflight");

    assert!(!summary.live_certified);
    assert_eq!(summary.live_certification_status, "blocked");
    assert_eq!(summary.missing_live_evidence.len(), EVIDENCE_FILES.len());
    assert!(
        summary
            .live_evidence
            .iter()
            .all(|entry| entry.status == "invalid"),
        "thin self-labeled live files must remain invalid: {:?}",
        summary.live_evidence
    );
}

#[test]
fn preflight_rejects_minimal_spoofed_live_evidence_files() {
    let temp_dir = tempdir().expect("temp dir");
    let evidence_dir = temp_dir
        .path()
        .join("target/reports/provider-certification/openrouter");
    fs::create_dir_all(&evidence_dir).expect("create provider evidence dir");
    for (file_name, _) in EVIDENCE_FILES {
        fs::write(
            evidence_dir.join(file_name),
            serde_json::json!({
                "provider": "openrouter",
                "status": "passed"
            })
            .to_string(),
        )
        .expect("write spoof evidence");
    }

    let summary = summarize_provider_certification(ProviderCertificationOptions {
        config: openrouter_config(),
        evidence_root: temp_dir.path().to_path_buf(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("provider certification preflight");

    assert!(!summary.live_certified);
    assert_eq!(summary.live_certification_status, "blocked");
    assert_eq!(summary.missing_live_evidence.len(), EVIDENCE_FILES.len());
    assert!(
        summary
            .live_evidence
            .iter()
            .all(|entry| entry.status == "invalid"),
        "minimal spoofed files must remain invalid: {:?}",
        summary.live_evidence
    );
}

#[test]
fn live_mode_without_stub_flag_is_rejected_by_api_and_cli() {
    let temp_dir = tempdir().expect("temp dir");
    let api_error = run_provider_live_certification(ProviderLiveCertificationOptions {
        config: openrouter_config(),
        evidence_root: temp_dir.path().to_path_buf(),
        mode: ProviderLiveCertificationMode::Live,
    })
    .expect_err("live mode should be unsupported until real checks exist");

    assert!(
        api_error.to_string().contains("--stub-evidence"),
        "API error should point callers to explicit stub evidence mode"
    );

    let binary = std::env::var("CARGO_BIN_EXE_provider_live_certification_run")
        .expect("provider_live_certification_run test binary path");
    let output = Command::new(binary)
        .arg("--evidence-root")
        .arg(temp_dir.path())
        .output()
        .expect("run provider live certification CLI");

    assert!(!output.status.success(), "default CLI live mode must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--stub-evidence"),
        "CLI must reject default live mode without pretending certification happened: {stderr}"
    );
}

fn openrouter_config() -> AppConfig {
    AppConfig {
        model_provider: ModelProviderKind::OpenRouter,
        model_config: ModelConfig::OpenRouter(OpenAiCompatibleConfig {
            base_url: "https://openrouter.example.test/api/v1?token=query-secret".to_string(),
            api_key: "openrouter-secret-key".to_string(),
            model: "openrouter/test-model".to_string(),
            timeout_ms: 30_000,
        }),
        ..Default::default()
    }
}
