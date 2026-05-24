use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::support::local_alpha_evidence::{
    LocalAlphaEvidenceOptions, LocalAlphaEvidenceSummary, summarize_local_alpha_evidence,
};
use crate::support::product_wording::{
    ClaimGateState, ProductClaimGuardInput, check_product_claims,
};
use crate::support::remote_team::{
    RemoteTeamCapabilityInventory, RemoteTeamCapabilityState, remote_team_capability_inventory,
    remote_team_security_gate_report,
};

#[derive(Debug, Clone)]
pub struct ProductReadinessOptions {
    pub evidence_root: PathBuf,
    pub release_candidate: String,
    pub output_json_path: Option<PathBuf>,
    pub output_markdown_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductReadinessSummary {
    pub generated_at: String,
    pub kind: &'static str,
    pub release_candidate: String,
    pub ready: bool,
    pub overall_status: &'static str,
    pub gates: Vec<ProductReadinessGate>,
    pub non_claims: Vec<String>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductReadinessGate {
    pub name: &'static str,
    pub status: &'static str,
    pub evidence_path: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseDecisionSummary {
    release_candidate: Option<String>,
    decision: Option<String>,
    human_reviewer: Option<String>,
    rollback_note: Option<String>,
}

pub fn summarize_product_readiness(
    options: ProductReadinessOptions,
) -> Result<ProductReadinessSummary> {
    let evidence_root = options.evidence_root;
    validate_release_candidate(&options.release_candidate)?;
    let local_alpha = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: evidence_root.clone(),
        output_json_path: None,
        output_markdown_path: None,
    })?;

    let mut gates = local_alpha_gates(&local_alpha);
    gates.push(release_decision_gate(
        &evidence_root,
        &options.release_candidate,
    ));
    gates.push(product_wording_gate(&options.release_candidate));
    gates.push(remote_team_gate());
    gates.push(security_auth_gate());

    let ready = gates.iter().all(|gate| gate.status == "satisfied");
    let overall_status = if ready { "ready_for_review" } else { "blocked" };
    let generated_at = Utc::now().to_rfc3339();
    let non_claims = vec![
        "not Local Alpha certification".to_string(),
        "not Windows parity evidence".to_string(),
        "not real fresh-machine evidence".to_string(),
        "not remote/team readiness".to_string(),
        "not GA or production-ready evidence".to_string(),
    ];
    let markdown = render_markdown(overall_status, &options.release_candidate, &gates);

    let summary = ProductReadinessSummary {
        generated_at,
        kind: "product_readiness_summary",
        release_candidate: options.release_candidate,
        ready,
        overall_status,
        gates,
        non_claims,
        markdown,
    };

    if let Some(path) = options.output_json_path {
        write_json(&path, &summary)?;
    }
    if let Some(path) = options.output_markdown_path {
        write_text(&path, &summary.markdown)?;
    }

    Ok(summary)
}

fn validate_release_candidate(candidate: &str) -> Result<()> {
    let allowed = !candidate.is_empty()
        && !candidate.starts_with('.')
        && !candidate.contains("..")
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'));

    if allowed {
        return Ok(());
    }

    Err(anyhow!(
        "candidate name must contain only letters, numbers, dot, underscore, and dash, and must not contain path traversal"
    ))
}

fn local_alpha_gates(summary: &LocalAlphaEvidenceSummary) -> Vec<ProductReadinessGate> {
    summary
        .gates
        .iter()
        .map(|gate| {
            let name = match gate.name {
                "first_run_bootstrap" => "real_fresh_machine",
                other => other,
            };
            let status = if gate.status == "satisfied" {
                "satisfied"
            } else {
                "blocked"
            };

            ProductReadinessGate {
                name,
                status,
                evidence_path: gate.evidence_path.clone(),
                reason: gate.reason.clone(),
            }
        })
        .collect()
}

fn release_decision_gate(root: &Path, release_candidate: &str) -> ProductReadinessGate {
    let relative = format!("target/reports/releases/{release_candidate}/release-decision.json");
    let path = root.join(&relative);
    let Ok(value) = read_json(&path) else {
        return gate(
            "release_decision",
            "blocked",
            Some(relative),
            "missing release decision summary",
        );
    };
    let Ok(summary) = serde_json::from_value::<ReleaseDecisionSummary>(value) else {
        return gate(
            "release_decision",
            "blocked",
            Some(relative),
            "release decision summary is not readable",
        );
    };

    let mut reasons = Vec::new();
    if summary.release_candidate.as_deref() != Some(release_candidate) {
        reasons.push("release candidate mismatch");
    }
    if summary.decision.as_deref() != Some("approved") {
        reasons.push("release decision is not approved");
    }
    if summary
        .human_reviewer
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("missing human reviewer");
    }
    if summary
        .rollback_note
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("missing rollback note");
    }

    if reasons.is_empty() {
        gate(
            "release_decision",
            "satisfied",
            Some(relative),
            "human release decision is approved",
        )
    } else {
        gate(
            "release_decision",
            "blocked",
            Some(relative),
            reasons.join("; "),
        )
    }
}

fn product_wording_gate(release_candidate: &str) -> ProductReadinessGate {
    let report = check_product_claims(ProductClaimGuardInput {
        text: release_candidate.to_string(),
        gate_state: ClaimGateState::default(),
    });

    if report.allowed {
        return gate(
            "product_wording",
            "satisfied",
            None,
            "candidate wording contains no blocked product claims",
        );
    }

    let claims = report
        .violations
        .iter()
        .map(|violation| violation.claim)
        .collect::<Vec<_>>()
        .join(", ");
    gate(
        "product_wording",
        "blocked",
        None,
        format!("blocked product claims require passed gates: {claims}"),
    )
}

fn remote_team_gate() -> ProductReadinessGate {
    let inventory = remote_team_capability_inventory();
    remote_team_readiness_gate_from_inventory(&inventory)
}

pub fn remote_team_readiness_gate_from_inventory(
    inventory: &RemoteTeamCapabilityInventory,
) -> ProductReadinessGate {
    let mut blockers = Vec::new();
    if !inventory.local_only {
        blockers.push("remote/team inventory is not local-only");
    }
    if inventory.support_bundle_upload_available {
        blockers.push("support bundle upload is available");
    }
    if inventory
        .capabilities
        .iter()
        .any(|capability| capability.write_capable_route_exposed)
    {
        blockers.push("write-capable remote/team routes are exposed");
    }
    if inventory
        .capabilities
        .iter()
        .any(|capability| capability.state != RemoteTeamCapabilityState::Blocked)
    {
        blockers.push("remote/team capability state is not blocked");
    }

    if blockers.is_empty() {
        return gate(
            "remote_team",
            "blocked",
            None,
            "remote/team capability inventory keeps remote and team features blocked until auth, audit, isolation, and rollback gates pass",
        );
    }

    gate("remote_team", "blocked", None, blockers.join("; "))
}

fn security_auth_gate() -> ProductReadinessGate {
    let report = remote_team_security_gate_report();
    if report.remote_writes_allowed {
        return gate(
            "security_auth",
            "satisfied",
            None,
            "remote security/auth gates are satisfied",
        );
    }

    let blockers = report.blocked_gate_names().join(", ");
    gate(
        "security_auth",
        "blocked",
        None,
        format!("blocked security gates: {blockers}"),
    )
}

fn render_markdown(
    overall_status: &str,
    release_candidate: &str,
    gates: &[ProductReadinessGate],
) -> String {
    let mut output = String::new();
    output.push_str("# Product Readiness Summary\n\n");
    output.push_str(&format!("- release_candidate: `{release_candidate}`\n"));
    output.push_str(&format!("- overall_status: `{overall_status}`\n\n"));
    output.push_str("| Gate | Status | Evidence | Reason |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for gate in gates {
        output.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            gate.name,
            gate.status,
            gate.evidence_path.as_deref().unwrap_or(""),
            gate.reason.replace('|', "\\|")
        ));
    }
    output
}

fn gate(
    name: &'static str,
    status: &'static str,
    evidence_path: Option<String>,
    reason: impl Into<String>,
) -> ProductReadinessGate {
    ProductReadinessGate {
        name,
        status,
        evidence_path,
        reason: reason.into(),
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).context("serialize product readiness summary")?,
    )
    .with_context(|| format!("write {}", path.display()))
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(path, value).with_context(|| format!("write {}", path.display()))
}
