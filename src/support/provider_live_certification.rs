use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::Serialize;

use crate::support::config::{AppConfig, ModelConfig};

const EVIDENCE_DIR_TEMPLATE: &str = "target/reports/provider-certification";
const EVIDENCE_FILES: &[(&str, &str)] = &[
    ("live-decision.json", "live_decision"),
    ("live-self-revision.json", "live_self_revision"),
    ("provider-error-handling.json", "provider_error_handling"),
    ("redaction-review.json", "redaction_review"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderLiveCertificationMode {
    StubEvidence,
    Live,
}

#[derive(Debug, Clone)]
pub struct ProviderLiveCertificationOptions {
    pub config: AppConfig,
    pub evidence_root: PathBuf,
    pub mode: ProviderLiveCertificationMode,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProviderLiveCertificationReport {
    pub generated_at: String,
    pub kind: &'static str,
    pub provider: String,
    pub status: &'static str,
    pub mode: &'static str,
    pub evidence_dir: String,
    pub generated_evidence: Vec<GeneratedEvidenceFile>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GeneratedEvidenceFile {
    pub evidence_kind: &'static str,
    pub status: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ProviderLiveEvidence {
    provider: String,
    status: &'static str,
    evidence_kind: &'static str,
    generated_at: String,
    local_only: bool,
    endpoint_shape: String,
    credential_configured: bool,
    mode: &'static str,
    non_claims: Vec<String>,
}

pub fn run_provider_live_certification(
    options: ProviderLiveCertificationOptions,
) -> Result<ProviderLiveCertificationReport> {
    match options.mode {
        ProviderLiveCertificationMode::StubEvidence => write_stub_evidence(options),
        ProviderLiveCertificationMode::Live => Err(anyhow!(
            "live provider certification is not implemented yet; pass --stub-evidence to generate explicit stub/simulated evidence"
        )),
    }
}

fn write_stub_evidence(
    options: ProviderLiveCertificationOptions,
) -> Result<ProviderLiveCertificationReport> {
    options
        .config
        .validate()
        .map_err(|error| anyhow!("provider configuration is not valid: {error}"))?;

    let provider = options.config.model_provider.as_str().to_string();
    let generated_at = Utc::now().to_rfc3339();
    let shape = provider_shape(&options.config);
    let evidence_dir = options
        .evidence_root
        .join(EVIDENCE_DIR_TEMPLATE)
        .join(&provider);
    fs::create_dir_all(&evidence_dir)
        .with_context(|| format!("create evidence directory {}", evidence_dir.display()))?;

    let non_claims = stub_non_claims();
    let mut generated_evidence = Vec::with_capacity(EVIDENCE_FILES.len());
    for (file_name, evidence_kind) in EVIDENCE_FILES {
        let evidence = ProviderLiveEvidence {
            provider: provider.clone(),
            status: "passed",
            evidence_kind,
            generated_at: generated_at.clone(),
            local_only: true,
            endpoint_shape: shape.endpoint_shape.clone(),
            credential_configured: shape.credential_configured,
            mode: "stub/simulated",
            non_claims: non_claims.clone(),
        };
        let path = evidence_dir.join(file_name);
        write_json(&path, &evidence)?;
        generated_evidence.push(GeneratedEvidenceFile {
            evidence_kind,
            status: "passed",
            path: path.to_string_lossy().to_string(),
        });
    }

    Ok(ProviderLiveCertificationReport {
        generated_at,
        kind: "provider_live_certification_evidence_run",
        provider,
        status: "passed",
        mode: "stub/simulated",
        evidence_dir: evidence_dir.to_string_lossy().to_string(),
        generated_evidence,
        non_claims,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderShape {
    endpoint_shape: String,
    credential_configured: bool,
}

fn provider_shape(config: &AppConfig) -> ProviderShape {
    match &config.model_config {
        ModelConfig::Mock => ProviderShape {
            endpoint_shape: "<none>".to_string(),
            credential_configured: false,
        },
        ModelConfig::OpenAiCompatible(provider) | ModelConfig::OpenRouter(provider) => {
            ProviderShape {
                endpoint_shape: endpoint_shape(&provider.base_url),
                credential_configured: !provider.api_key.trim().is_empty(),
            }
        }
    }
}

fn endpoint_shape(base_url: &str) -> String {
    let Ok(parsed) = reqwest::Url::parse(base_url) else {
        return "<redacted>".to_string();
    };
    let Some(host) = parsed.host_str() else {
        return "<redacted>".to_string();
    };

    let mut shaped = format!("{}://{}", parsed.scheme(), host);
    if let Some(port) = parsed.port() {
        shaped.push(':');
        shaped.push_str(&port.to_string());
    }
    if parsed.path() != "/" {
        shaped.push_str("/<redacted-path>");
    }
    shaped
}

fn stub_non_claims() -> Vec<String> {
    vec![
        "stub/simulated evidence only".to_string(),
        "not real live provider evidence".to_string(),
        "not provider endpoint reachability evidence".to_string(),
        "not provider decision quality evidence".to_string(),
        "not provider self-revision quality evidence".to_string(),
        "does not include request bodies, response bodies, API keys, URL userinfo, URL paths, query values, or provider-native payloads".to_string(),
    ]
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value)
            .context("serialize provider live certification evidence")?,
    )
    .with_context(|| format!("write {}", path.display()))
}
