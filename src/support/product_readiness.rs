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
    pub external_blockers: Vec<ProductReadinessBlocker>,
    pub human_blockers: Vec<ProductReadinessBlocker>,
    pub unimplemented_capability_blockers: Vec<ProductReadinessBlocker>,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductReadinessBlocker {
    pub subject: &'static str,
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

#[derive(Debug, Deserialize)]
struct ReleaseCompatibilityMatrix {
    kind: Option<String>,
    candidate: Option<String>,
    local_only: Option<bool>,
    rows: Option<Vec<ReleaseCompatibilityRow>>,
}

#[derive(Debug, Deserialize)]
struct ReleaseCompatibilityRow {
    platform: Option<String>,
    result: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseBoundariesSummary {
    kind: Option<String>,
    candidate: Option<String>,
    local_only: Option<bool>,
    product_boundary: Option<String>,
    external_blockers: Option<Vec<ReleaseBoundaryBlocker>>,
    human_blockers: Option<Vec<ReleaseBoundaryBlocker>>,
    unimplemented_capability_blockers: Option<Vec<ReleaseBoundaryBlocker>>,
}

#[derive(Debug, Deserialize)]
struct ReleaseBoundaryBlocker {
    subject: Option<String>,
    status: Option<String>,
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
    gates.push(release_engineering_gate(
        &evidence_root,
        &options.release_candidate,
    ));
    gates.push(product_wording_gate(&options.release_candidate));
    gates.push(remote_team_gate());
    gates.push(security_auth_gate());

    let ready = gates.iter().all(|gate| gate.status == "satisfied");
    let overall_status = if ready { "ready_for_review" } else { "blocked" };
    let external_blockers = external_blockers(&gates);
    let human_blockers = human_blockers(&gates);
    let unimplemented_capability_blockers = unimplemented_capability_blockers(&gates);
    let generated_at = Utc::now().to_rfc3339();
    let non_claims = vec![
        "not Local Alpha certification".to_string(),
        "not Windows parity evidence".to_string(),
        "not real fresh-machine evidence".to_string(),
        "not remote/team readiness".to_string(),
        "not GA or production-ready evidence".to_string(),
        "not physics-informed runtime / solver / controller / scientific validation evidence"
            .to_string(),
    ];
    let markdown = render_markdown(overall_status, &options.release_candidate, &gates);

    let summary = ProductReadinessSummary {
        generated_at,
        kind: "product_readiness_summary",
        release_candidate: options.release_candidate,
        ready,
        overall_status,
        gates,
        external_blockers,
        human_blockers,
        unimplemented_capability_blockers,
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

fn release_engineering_gate(root: &Path, release_candidate: &str) -> ProductReadinessGate {
    let relative_dir = format!("target/reports/releases/{release_candidate}");
    let matrix_relative = format!("{relative_dir}/compatibility-matrix.json");
    let boundaries_relative = format!("{relative_dir}/release-boundaries.json");
    let matrix_path = root.join(&matrix_relative);
    let boundaries_path = root.join(&boundaries_relative);

    let mut missing = Vec::new();
    if !matrix_path.is_file() {
        missing.push("compatibility-matrix.json");
    }
    if !boundaries_path.is_file() {
        missing.push("release-boundaries.json");
    }
    if !missing.is_empty() {
        return gate(
            "release_engineering",
            "blocked",
            Some(relative_dir),
            format!(
                "missing source-only release engineering artifacts: {}",
                missing.join(", ")
            ),
        );
    }

    let Ok(matrix_value) = read_json(&matrix_path) else {
        return gate(
            "release_engineering",
            "blocked",
            Some(matrix_relative),
            "source-only release engineering artifact is not readable",
        );
    };
    let Ok(boundaries_value) = read_json(&boundaries_path) else {
        return gate(
            "release_engineering",
            "blocked",
            Some(boundaries_relative),
            "source-only release engineering artifact is not readable",
        );
    };
    let Ok(matrix) = serde_json::from_value::<ReleaseCompatibilityMatrix>(matrix_value) else {
        return gate(
            "release_engineering",
            "blocked",
            Some(matrix_relative),
            "compatibility matrix is not readable",
        );
    };
    let Ok(boundaries) = serde_json::from_value::<ReleaseBoundariesSummary>(boundaries_value)
    else {
        return gate(
            "release_engineering",
            "blocked",
            Some(boundaries_relative),
            "release boundaries artifact is not readable",
        );
    };

    let mut reasons = Vec::new();
    if matrix.kind.as_deref() != Some("release_compatibility_matrix") {
        reasons.push("compatibility matrix kind mismatch");
    }
    if matrix.candidate.as_deref() != Some(release_candidate) {
        reasons.push("compatibility matrix candidate mismatch");
    }
    if matrix.local_only != Some(true) {
        reasons.push("compatibility matrix is not local-only");
    }
    let rows = matrix.rows.as_deref().unwrap_or(&[]);
    if !rows.iter().any(|row| {
        row.platform
            .as_deref()
            .is_some_and(|platform| platform != "Windows")
            && row.result.as_deref() == Some("passed")
    }) {
        reasons.push("compatibility matrix has no local passed row");
    }
    if !rows.iter().any(|row| {
        row.platform.as_deref() == Some("Windows") && row.result.as_deref() == Some("not_checked")
    }) {
        reasons.push("compatibility matrix does not keep Windows parity not_checked");
    }

    if boundaries.kind.as_deref() != Some("release_boundaries") {
        reasons.push("release boundaries kind mismatch");
    }
    if boundaries.candidate.as_deref() != Some(release_candidate) {
        reasons.push("release boundaries candidate mismatch");
    }
    if boundaries.local_only != Some(true) {
        reasons.push("release boundaries are not local-only");
    }
    if !boundaries
        .product_boundary
        .as_deref()
        .unwrap_or_default()
        .contains("local Rust MCP stdio memory MVP / technical demo")
    {
        reasons.push("release boundaries do not preserve local MVP product boundary");
    }
    if !has_blocker(
        boundaries.external_blockers.as_deref(),
        "fresh_machine",
        "blocked",
    ) {
        reasons.push("fresh-machine external blocker is missing");
    }
    if !has_blocker(
        boundaries.external_blockers.as_deref(),
        "windows_parity",
        "not_checked",
    ) {
        reasons.push("Windows parity not_checked blocker is missing");
    }
    if !has_blocker(
        boundaries.human_blockers.as_deref(),
        "release_decision",
        "required",
    ) {
        reasons.push("human release decision blocker is missing");
    }
    if !has_blocker(
        boundaries.unimplemented_capability_blockers.as_deref(),
        "remote_team",
        "blocked",
    ) {
        reasons.push("remote/team blocker is missing");
    }
    if !has_blocker(
        boundaries.unimplemented_capability_blockers.as_deref(),
        "security_auth",
        "blocked",
    ) {
        reasons.push("security/auth blocker is missing");
    }
    if !has_blocker(
        boundaries.unimplemented_capability_blockers.as_deref(),
        "daemon_writes",
        "blocked",
    ) {
        reasons.push("daemon write blocker is missing");
    }
    if !has_blocker(
        boundaries.unimplemented_capability_blockers.as_deref(),
        "release_packaging",
        "blocked",
    ) {
        reasons.push("release packaging blocker is missing");
    }

    if reasons.is_empty() {
        gate(
            "release_engineering",
            "satisfied",
            Some(relative_dir),
            "source-only release engineering artifacts are present; Windows, fresh-machine, human decision, and packaging remain explicit blockers",
        )
    } else {
        gate(
            "release_engineering",
            "blocked",
            Some(relative_dir),
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

fn external_blockers(gates: &[ProductReadinessGate]) -> Vec<ProductReadinessBlocker> {
    gates
        .iter()
        .filter_map(|gate| match gate.name {
            "real_fresh_machine" if gate.status != "satisfied" => Some(product_blocker(
                "fresh_machine",
                gate.status,
                gate.evidence_path.clone(),
                gate.reason.clone(),
            )),
            "windows_parity" if gate.status != "satisfied" => Some(product_blocker(
                "windows_parity",
                gate.status,
                gate.evidence_path.clone(),
                gate.reason.clone(),
            )),
            _ => None,
        })
        .collect()
}

fn human_blockers(gates: &[ProductReadinessGate]) -> Vec<ProductReadinessBlocker> {
    gates
        .iter()
        .filter_map(|gate| match gate.name {
            "release_decision" if gate.status != "satisfied" => Some(product_blocker(
                "release_decision",
                gate.status,
                gate.evidence_path.clone(),
                gate.reason.clone(),
            )),
            _ => None,
        })
        .collect()
}

fn unimplemented_capability_blockers(
    gates: &[ProductReadinessGate],
) -> Vec<ProductReadinessBlocker> {
    let mut blockers = gates
        .iter()
        .filter_map(|gate| match gate.name {
            "remote_team" | "security_auth" if gate.status != "satisfied" => Some(product_blocker(
                gate.name,
                gate.status,
                gate.evidence_path.clone(),
                gate.reason.clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    blockers.push(product_blocker(
        "daemon_writes",
        "blocked",
        None,
        "daemon write capability remains disabled; observe-only diagnostics do not create durable semantic writes or background autonomy",
    ));
    blockers.push(product_blocker(
        "release_packaging",
        "blocked",
        None,
        "binary packaging, installer, service manager, and auto-updater evidence are not implemented",
    ));
    blockers
}

fn has_blocker(
    blockers: Option<&[ReleaseBoundaryBlocker]>,
    expected_subject: &str,
    expected_status: &str,
) -> bool {
    blockers.unwrap_or_default().iter().any(|blocker| {
        blocker.subject.as_deref() == Some(expected_subject)
            && blocker.status.as_deref() == Some(expected_status)
    })
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

fn product_blocker(
    subject: &'static str,
    status: &'static str,
    evidence_path: Option<String>,
    reason: impl Into<String>,
) -> ProductReadinessBlocker {
    ProductReadinessBlocker {
        subject,
        status,
        evidence_path,
        reason: reason.into(),
    }
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
