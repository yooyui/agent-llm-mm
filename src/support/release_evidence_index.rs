use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;

use crate::support::local_alpha_evidence::{
    LocalAlphaEvidenceOptions, LocalAlphaGateSummary, summarize_local_alpha_evidence,
};
use crate::support::product_readiness::{
    ProductReadinessBlocker, ProductReadinessGate, ProductReadinessOptions,
    summarize_product_readiness,
};

#[derive(Debug, Clone)]
pub struct ReleaseEvidenceIndexOptions {
    pub evidence_root: PathBuf,
    pub release_candidate: String,
    pub output_json_path: Option<PathBuf>,
    pub output_markdown_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseEvidenceIndex {
    pub generated_at: String,
    pub kind: &'static str,
    pub release_candidate: String,
    pub local_only: bool,
    pub ready_for_human_review: bool,
    pub missing_required_count: usize,
    pub blocked_gate_count: usize,
    pub entries: Vec<ReleaseEvidenceIndexEntry>,
    pub non_claims: Vec<String>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseEvidenceIndexEntry {
    pub name: &'static str,
    pub status: &'static str,
    pub source: &'static str,
    pub evidence_path: Option<String>,
    pub reason: String,
}

pub fn build_release_evidence_index(
    options: ReleaseEvidenceIndexOptions,
) -> Result<ReleaseEvidenceIndex> {
    let evidence_root = options.evidence_root;
    let local_alpha = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: evidence_root.clone(),
        output_json_path: None,
        output_markdown_path: None,
    })?;
    let product_readiness = summarize_product_readiness(ProductReadinessOptions {
        evidence_root,
        release_candidate: options.release_candidate.clone(),
        output_json_path: None,
        output_markdown_path: None,
    })?;

    let mut entries = Vec::new();
    let mut local_names = BTreeSet::new();
    for gate in &local_alpha.gates {
        local_names.insert(gate.name);
        entries.push(local_alpha_entry(gate));
    }

    let product_to_local = product_to_local_gate_names();
    for gate in &product_readiness.gates {
        if product_to_local
            .get(gate.name)
            .is_some_and(|local_name| local_names.contains(local_name))
        {
            continue;
        }
        entries.push(product_gate_entry(gate));
    }

    let existing_names = entries
        .iter()
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    for blocker in &product_readiness.unimplemented_capability_blockers {
        if !existing_names.contains(blocker.subject) {
            entries.push(blocked_capability_entry(blocker));
        }
    }

    let missing_required_count = entries
        .iter()
        .filter(|entry| matches!(entry.status, "missing" | "not_verified"))
        .count();
    let blocked_gate_count = entries
        .iter()
        .filter(|entry| entry.status == "blocked")
        .count();
    let ready_for_human_review = entries.iter().all(|entry| entry.status == "present");
    let generated_at = Utc::now().to_rfc3339();
    let non_claims = non_claims(product_readiness.non_claims);
    let markdown = render_markdown(&options.release_candidate, &entries, &non_claims);

    let index = ReleaseEvidenceIndex {
        generated_at,
        kind: "release_evidence_index",
        release_candidate: options.release_candidate,
        local_only: true,
        ready_for_human_review,
        missing_required_count,
        blocked_gate_count,
        entries,
        non_claims,
        markdown,
    };

    if let Some(path) = options.output_json_path {
        write_json(&path, &index)?;
    }
    if let Some(path) = options.output_markdown_path {
        write_text(&path, &index.markdown)?;
    }

    Ok(index)
}

fn local_alpha_entry(gate: &LocalAlphaGateSummary) -> ReleaseEvidenceIndexEntry {
    let status = match gate.status {
        "satisfied" => "present",
        "open" => "missing",
        "not_verified" => "not_verified",
        _ => "not_verified",
    };
    entry(
        gate.name,
        status,
        "local_alpha_evidence",
        gate.evidence_path.clone(),
        gate.reason.clone(),
    )
}

fn product_gate_entry(gate: &ProductReadinessGate) -> ReleaseEvidenceIndexEntry {
    let status = match gate.status {
        "satisfied" => "present",
        "blocked" => "blocked",
        "not_verified" => "not_verified",
        _ => "missing",
    };
    entry(
        gate.name,
        status,
        "product_readiness",
        gate.evidence_path.clone(),
        gate.reason.clone(),
    )
}

fn blocked_capability_entry(blocker: &ProductReadinessBlocker) -> ReleaseEvidenceIndexEntry {
    entry(
        blocker.subject,
        "blocked",
        "product_readiness",
        blocker.evidence_path.clone(),
        blocker.reason.clone(),
    )
}

fn product_to_local_gate_names() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("product_smoke", "product_smoke"),
        ("real_fresh_machine", "first_run_bootstrap"),
        ("first_run_simulation", "first_run_simulation"),
        ("windows_parity", "windows_parity"),
        ("support_bundle", "support_bundle"),
        ("local_read_only_boundary", "local_read_only_boundary"),
    ])
}

fn non_claims(mut product_non_claims: Vec<String>) -> Vec<String> {
    let mut claims = vec![
        "not release approval".to_string(),
        "not a release decision".to_string(),
        "not remote/team readiness".to_string(),
        "not binary packaging, installer, service manager, or auto-updater evidence".to_string(),
        "not Beta, GA, or production-ready evidence".to_string(),
    ];
    claims.append(&mut product_non_claims);
    claims.sort();
    claims.dedup();
    claims
}

fn render_markdown(
    release_candidate: &str,
    entries: &[ReleaseEvidenceIndexEntry],
    non_claims: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("# Release Evidence Index\n\n");
    output.push_str(&format!("- release_candidate: `{release_candidate}`\n"));
    output.push_str("- local_only: `true`\n\n");
    output.push_str("| Entry | Status | Source | Evidence | Reason |\n");
    output.push_str("| --- | --- | --- | --- | --- |\n");
    for entry in entries {
        output.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} |\n",
            entry.name,
            entry.status,
            entry.source,
            entry.evidence_path.as_deref().unwrap_or(""),
            entry.reason.replace('|', "\\|")
        ));
    }
    output.push_str("\n## Non-Claims\n\n");
    for claim in non_claims {
        output.push_str(&format!("- {claim}\n"));
    }
    output
}

fn entry(
    name: &'static str,
    status: &'static str,
    source: &'static str,
    evidence_path: Option<String>,
    reason: String,
) -> ReleaseEvidenceIndexEntry {
    ReleaseEvidenceIndexEntry {
        name,
        status,
        source,
        evidence_path,
        reason,
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).context("serialize release evidence index")?,
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
