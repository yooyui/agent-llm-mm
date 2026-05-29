use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::Serialize;

use crate::support::local_alpha_evidence::{
    LocalAlphaEvidenceOptions, LocalAlphaGateSummary, summarize_local_alpha_evidence,
};

#[derive(Debug, Clone)]
pub struct ReleaseDecisionOptions {
    pub evidence_root: PathBuf,
    pub release_candidate: String,
    pub request: ReleaseDecisionRequest,
    pub output_json_path: Option<PathBuf>,
    pub output_markdown_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseDecisionRequest {
    pub decision: String,
    pub human_reviewer: Option<String>,
    pub rollback_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseDecisionArtifact {
    pub generated_at: String,
    pub kind: &'static str,
    pub release_candidate: String,
    pub decision: String,
    pub approved: bool,
    pub human_reviewer: Option<String>,
    pub rollback_note: Option<String>,
    pub evidence_status: &'static str,
    pub evidence_root: String,
    pub open_gates: Vec<ReleaseDecisionGate>,
    pub external_blockers: Vec<ReleaseDecisionBlocker>,
    pub human_blockers: Vec<ReleaseDecisionBlocker>,
    pub unimplemented_capability_blockers: Vec<ReleaseDecisionBlocker>,
    pub non_claims: Vec<String>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseDecisionGate {
    pub name: &'static str,
    pub status: &'static str,
    pub evidence_path: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseDecisionBlocker {
    pub subject: &'static str,
    pub status: &'static str,
    pub evidence_path: Option<String>,
    pub reason: String,
}

pub fn write_release_decision(options: ReleaseDecisionOptions) -> Result<ReleaseDecisionArtifact> {
    let evidence = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: options.evidence_root.clone(),
        output_json_path: None,
        output_markdown_path: None,
    })?;
    let decision = normalize_decision(&options.request.decision)?;
    let approved = decision == "approved";
    let human_reviewer = normalized_optional(options.request.human_reviewer);
    let rollback_note = normalized_optional(options.request.rollback_note);

    if approved && evidence.overall_status != "ready_for_human_review" {
        return Err(anyhow!(
            "cannot approve release decision while Local Alpha evidence status is {}",
            evidence.overall_status
        ));
    }
    if approved && human_reviewer.is_none() {
        return Err(anyhow!(
            "cannot approve release decision without human_reviewer"
        ));
    }
    if approved && rollback_note.is_none() {
        return Err(anyhow!(
            "cannot approve release decision without rollback_note"
        ));
    }

    let open_gates = evidence
        .gates
        .iter()
        .filter(|gate| gate.status != "satisfied")
        .map(release_decision_gate)
        .collect::<Vec<_>>();
    let external_blockers = external_blockers(&open_gates);
    let human_blockers = human_blockers(
        decision,
        human_reviewer.as_deref(),
        rollback_note.as_deref(),
    );
    let unimplemented_capability_blockers = unimplemented_capability_blockers();
    let non_claims = non_claims(approved);
    let generated_at = Utc::now().to_rfc3339();
    let release_candidate = options.release_candidate;
    let evidence_root = options.evidence_root.display().to_string();
    let markdown = render_markdown(
        &release_candidate,
        decision,
        approved,
        evidence.overall_status,
        &open_gates,
        human_reviewer.as_deref(),
        rollback_note.as_deref(),
    );

    let artifact = ReleaseDecisionArtifact {
        generated_at,
        kind: "release_decision",
        release_candidate,
        decision: decision.to_string(),
        approved,
        human_reviewer,
        rollback_note,
        evidence_status: evidence.overall_status,
        evidence_root,
        open_gates,
        external_blockers,
        human_blockers,
        unimplemented_capability_blockers,
        non_claims,
        markdown,
    };

    if let Some(path) = options.output_json_path {
        write_json(&path, &artifact)?;
    }
    if let Some(path) = options.output_markdown_path {
        write_text(&path, &artifact.markdown)?;
    }

    Ok(artifact)
}

fn normalize_decision(decision: &str) -> Result<&'static str> {
    match decision.trim() {
        "blocked" => Ok("blocked"),
        "approved" => Ok("approved"),
        other => Err(anyhow!(
            "unsupported release decision `{other}`; expected blocked or approved"
        )),
    }
}

fn normalized_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn release_decision_gate(gate: &LocalAlphaGateSummary) -> ReleaseDecisionGate {
    ReleaseDecisionGate {
        name: gate.name,
        status: gate.status,
        evidence_path: gate.evidence_path.clone(),
        reason: gate.reason.clone(),
    }
}

fn external_blockers(open_gates: &[ReleaseDecisionGate]) -> Vec<ReleaseDecisionBlocker> {
    open_gates
        .iter()
        .filter_map(|gate| match gate.name {
            "first_run_bootstrap" => Some(release_blocker(
                "fresh_machine",
                gate.status,
                gate.evidence_path.clone(),
                gate.reason.clone(),
            )),
            "windows_parity" => Some(release_blocker(
                "windows_parity",
                gate.status,
                gate.evidence_path.clone(),
                gate.reason.clone(),
            )),
            _ => None,
        })
        .collect()
}

fn human_blockers(
    decision: &str,
    human_reviewer: Option<&str>,
    rollback_note: Option<&str>,
) -> Vec<ReleaseDecisionBlocker> {
    let mut blockers = Vec::new();
    if decision != "approved" {
        blockers.push(release_blocker(
            "release_approval",
            "blocked",
            None,
            format!("decision is {decision}; human release approval has not been recorded"),
        ));
    }
    if human_reviewer.is_none() {
        blockers.push(release_blocker(
            "human_reviewer",
            "blocked",
            None,
            "human reviewer has not been recorded",
        ));
    }
    if rollback_note.is_none() {
        blockers.push(release_blocker(
            "rollback_note",
            "blocked",
            None,
            "rollback note has not been recorded",
        ));
    }
    blockers
}

fn unimplemented_capability_blockers() -> Vec<ReleaseDecisionBlocker> {
    vec![
        release_blocker(
            "remote_team",
            "blocked",
            None,
            "remote/team capability remains unimplemented and blocked by auth, audit, isolation, and rollback gates",
        ),
        release_blocker(
            "security_auth",
            "blocked",
            None,
            "remote auth, authorization, audit, rate limit, tenant isolation, and rollback implementation is absent",
        ),
        release_blocker(
            "daemon_writes",
            "blocked",
            None,
            "daemon write capability remains disabled; observe-only diagnostics do not create durable semantic writes or background autonomy",
        ),
        release_blocker(
            "release_packaging",
            "blocked",
            None,
            "binary packaging, installer, service manager, and auto-updater evidence are not implemented",
        ),
    ]
}

fn non_claims(approved: bool) -> Vec<String> {
    let mut claims = vec![
        "not binary packaging".to_string(),
        "not installer evidence".to_string(),
        "not Windows parity evidence unless a Windows gate is satisfied".to_string(),
        "not real fresh-machine evidence unless that gate is satisfied".to_string(),
        "not remote/team readiness".to_string(),
        "not Beta, GA, or production-ready evidence".to_string(),
    ];
    if !approved {
        claims.insert(0, "not release approval".to_string());
    }
    claims
}

fn release_blocker(
    subject: &'static str,
    status: &'static str,
    evidence_path: Option<String>,
    reason: impl Into<String>,
) -> ReleaseDecisionBlocker {
    ReleaseDecisionBlocker {
        subject,
        status,
        evidence_path,
        reason: reason.into(),
    }
}

fn render_markdown(
    release_candidate: &str,
    decision: &str,
    approved: bool,
    evidence_status: &str,
    open_gates: &[ReleaseDecisionGate],
    human_reviewer: Option<&str>,
    rollback_note: Option<&str>,
) -> String {
    let mut output = String::new();
    output.push_str("# Release Decision\n\n");
    output.push_str(&format!("- release_candidate: `{release_candidate}`\n"));
    output.push_str(&format!("- decision: `{decision}`\n"));
    output.push_str(&format!("- approved: `{approved}`\n"));
    output.push_str(&format!("- evidence_status: `{evidence_status}`\n"));
    output.push_str(&format!(
        "- human_reviewer: `{}`\n",
        human_reviewer.unwrap_or("")
    ));
    output.push_str(&format!(
        "- rollback_note: `{}`\n\n",
        rollback_note.unwrap_or("")
    ));

    output.push_str("| Open Gate | Status | Evidence | Reason |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    if open_gates.is_empty() {
        output.push_str("|  |  |  | no open Local Alpha evidence gates at generation time |\n");
    } else {
        for gate in open_gates {
            output.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                gate.name,
                gate.status,
                gate.evidence_path.as_deref().unwrap_or(""),
                gate.reason.replace('|', "\\|")
            ));
        }
    }
    output.push_str(
        "\nThis artifact records a local source-only release decision. It does not create tags, packages, uploads, remote/team readiness, Beta, GA, or production-ready evidence.\n",
    );
    output
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).context("serialize release decision artifact")?,
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
