use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const PRODUCT_SMOKE_LATEST: &[&str] = &[
    "target/reports/self-revision-demo/latest/doctor.json",
    "target/reports/self-revision-demo/latest/snapshot-before.json",
    "target/reports/self-revision-demo/latest/snapshot-after.json",
    "target/reports/self-revision-demo/latest/decision-before.json",
    "target/reports/self-revision-demo/latest/decision-after.json",
    "target/reports/self-revision-demo/latest/timeline.json",
    "target/reports/self-revision-demo/latest/sqlite-summary.json",
    "target/reports/self-revision-demo/latest/report.md",
];

const ALLOWED_SUPPORT_BUNDLE_FILES: &[&str] = &[
    "manifest.json",
    "doctor.json",
    "config-shape.json",
    "operation-summaries.json",
    "release-metadata.json",
    "product-smoke-summary.json",
    "local-log-excerpts.json",
];

const FIRST_RUN_BOUNDARY_INCOMPLETE: &str = "first-run boundary evidence is incomplete";

#[derive(Debug, Clone)]
pub struct LocalAlphaEvidenceOptions {
    pub evidence_root: PathBuf,
    pub output_json_path: Option<PathBuf>,
    pub output_markdown_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LocalAlphaEvidenceSummary {
    pub generated_at: String,
    pub kind: &'static str,
    pub overall_status: &'static str,
    pub local_only: bool,
    pub summary_boundary: &'static str,
    pub gates: Vec<LocalAlphaGateSummary>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LocalAlphaGateSummary {
    pub name: &'static str,
    pub status: &'static str,
    pub evidence_path: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
struct FirstRunSummary {
    doctor_status: Option<String>,
    real_fresh_machine_evidence: Option<bool>,
    local_only: Option<bool>,
    self_revision_write_path: Option<String>,
    daemon_enabled: Option<bool>,
    daemon_writes_allowed: Option<bool>,
    started_serve: Option<bool>,
    ran_product_smoke: Option<bool>,
    sqlite_database_exists: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct WindowsParitySummary {
    status: Option<String>,
    runner: Option<String>,
    platform: Option<String>,
    runtime_parity: Option<bool>,
}

pub fn summarize_local_alpha_evidence(
    options: LocalAlphaEvidenceOptions,
) -> Result<LocalAlphaEvidenceSummary> {
    let evidence_root = options.evidence_root;
    let gates = vec![
        product_smoke_gate(&evidence_root),
        first_run_bootstrap_gate(&evidence_root),
        windows_parity_gate(&evidence_root),
        support_bundle_gate(&evidence_root),
        boundary_guard_gate(&evidence_root),
    ];
    let overall_status = overall_status(&gates);
    let generated_at = Utc::now().to_rfc3339();
    let markdown = render_markdown(overall_status, &gates);

    let summary = LocalAlphaEvidenceSummary {
        generated_at,
        kind: "local_alpha_evidence_summary",
        overall_status,
        local_only: true,
        summary_boundary: "read-only gate status summary; not automatic certification; run_reflection remains the only durable write path",
        gates,
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

fn product_smoke_gate(root: &Path) -> LocalAlphaGateSummary {
    let missing = missing_paths(root, PRODUCT_SMOKE_LATEST);
    let evidence_path = "target/reports/self-revision-demo/latest";
    if missing.is_empty() {
        gate(
            "product_smoke",
            "satisfied",
            Some(evidence_path),
            "product smoke latest evidence artifacts are present",
        )
    } else {
        gate(
            "product_smoke",
            "open",
            Some(evidence_path),
            format!(
                "missing or empty product smoke latest evidence: {}",
                missing.join(", ")
            ),
        )
    }
}

fn first_run_bootstrap_gate(root: &Path) -> LocalAlphaGateSummary {
    let Some((relative, path)) = first_existing_file(
        root,
        &[
            "first-run-bootstrap/summary.json",
            "target/first-run-bootstrap-smoke/local-alpha-gate/summary.json",
        ],
    ) else {
        return gate(
            "first_run_bootstrap",
            "open",
            Some("first-run-bootstrap/summary.json"),
            "missing first-run bootstrap summary",
        );
    };
    let Ok(value) = read_json(&path) else {
        return gate(
            "first_run_bootstrap",
            "open",
            Some(relative),
            "missing first-run bootstrap summary",
        );
    };
    let Ok(summary) = serde_json::from_value::<FirstRunSummary>(value) else {
        return gate(
            "first_run_bootstrap",
            "open",
            Some(relative),
            "first-run bootstrap summary is not readable",
        );
    };

    let mut reasons = Vec::new();
    if summary.doctor_status.as_deref() != Some("ok") {
        reasons.push("doctor_status is not ok");
    }
    if summary.real_fresh_machine_evidence != Some(true) {
        reasons.push("real fresh-machine evidence is false");
    }
    if summary.local_only != Some(true) {
        reasons.push("local_only is not true");
    }
    if summary.self_revision_write_path.as_deref() != Some("run_reflection") {
        reasons.push("self_revision_write_path is not run_reflection");
    }
    if summary.daemon_enabled == Some(true) || summary.daemon_writes_allowed == Some(true) {
        reasons.push("daemon write path is enabled");
    }
    if summary.started_serve == Some(true) {
        reasons.push("first-run smoke started serve");
    }
    if summary.ran_product_smoke == Some(true) {
        reasons.push("first-run smoke ran product smoke");
    }
    if summary.sqlite_database_exists != Some(true) {
        reasons.push("sqlite_database_exists is not true");
    }

    if reasons.is_empty() {
        gate(
            "first_run_bootstrap",
            "satisfied",
            Some(relative),
            "real fresh-machine first-run evidence is present and local-only",
        )
    } else {
        gate(
            "first_run_bootstrap",
            "open",
            Some(relative),
            reasons.join("; "),
        )
    }
}

fn windows_parity_gate(root: &Path) -> LocalAlphaGateSummary {
    let Some((relative, path)) = first_existing_file(
        root,
        &[
            "windows-parity/summary.json",
            "target/windows-parity/local-alpha-gate/summary.json",
        ],
    ) else {
        return gate(
            "windows_parity",
            "not_verified",
            Some("windows-parity/summary.json"),
            "missing Windows runtime parity evidence",
        );
    };
    let Ok(value) = read_json(&path) else {
        return gate(
            "windows_parity",
            "not_verified",
            Some(relative),
            "missing Windows runtime parity evidence",
        );
    };
    let Ok(summary) = serde_json::from_value::<WindowsParitySummary>(value) else {
        return gate(
            "windows_parity",
            "not_verified",
            Some(relative),
            "Windows runtime parity summary is not readable",
        );
    };

    let runner_or_platform_is_windows = summary
        .runner
        .as_deref()
        .or(summary.platform.as_deref())
        .is_some_and(|value| value.to_ascii_lowercase().contains("windows"));

    if summary.status.as_deref() == Some("verified")
        && summary.runtime_parity == Some(true)
        && runner_or_platform_is_windows
    {
        gate(
            "windows_parity",
            "satisfied",
            Some(relative),
            "Windows runtime parity evidence is verified",
        )
    } else {
        gate(
            "windows_parity",
            "not_verified",
            Some(relative),
            if runner_or_platform_is_windows {
                "Windows runtime parity evidence is not verified"
            } else {
                "missing Windows runner or platform evidence"
            },
        )
    }
}

fn support_bundle_gate(root: &Path) -> LocalAlphaGateSummary {
    let Some((relative, path)) = first_existing_dir(
        root,
        &["support-bundle", "target/support-bundles/local-alpha-gate"],
    ) else {
        return gate(
            "support_bundle",
            "open",
            Some("support-bundle"),
            "missing support bundle directory",
        );
    };

    let allowed = ALLOWED_SUPPORT_BUNDLE_FILES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut unexpected = Vec::new();
    let mut present = BTreeSet::new();
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(_) => {
            return gate(
                "support_bundle",
                "open",
                Some(relative),
                "support bundle directory is unreadable",
            );
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let Ok(file_type) = entry.file_type() else {
            unexpected.push(entry.file_name().to_string_lossy().into_owned());
            continue;
        };
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !file_type.is_file() || !allowed.contains(file_name.as_str()) {
            unexpected.push(file_name);
            continue;
        }
        present.insert(file_name);
    }

    let missing = ALLOWED_SUPPORT_BUNDLE_FILES
        .iter()
        .filter(|file| !present.contains(**file))
        .copied()
        .collect::<Vec<_>>();

    let validation_reasons = validate_support_bundle(&path);

    if unexpected.is_empty() && missing.is_empty() && validation_reasons.is_empty() {
        gate(
            "support_bundle",
            "satisfied",
            Some(relative),
            "allowed local diagnostic files present",
        )
    } else {
        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(format!(
                "missing support bundle diagnostics: {}",
                missing.join(", ")
            ));
        }
        if !unexpected.is_empty() {
            reasons.push(format!(
                "unexpected support bundle files: {}",
                unexpected.join(", ")
            ));
        }
        reasons.extend(validation_reasons);
        gate("support_bundle", "open", Some(relative), reasons.join("; "))
    }
}

fn boundary_guard_gate(root: &Path) -> LocalAlphaGateSummary {
    let Some((relative, path)) = first_existing_file(
        root,
        &[
            "first-run-bootstrap/summary.json",
            "target/first-run-bootstrap-smoke/local-alpha-gate/summary.json",
        ],
    ) else {
        return gate(
            "local_read_only_boundary",
            "not_verified",
            Some("first-run-bootstrap/summary.json"),
            "missing first-run boundary evidence",
        );
    };
    let mut reasons = Vec::new();

    let Ok(value) = read_json(&path) else {
        return gate(
            "local_read_only_boundary",
            "not_verified",
            Some(relative),
            "first-run boundary evidence is unreadable",
        );
    };
    if value.get("started_serve").and_then(Value::as_bool) == Some(true) {
        reasons.push("first-run evidence says the runtime server was launched");
    } else if value
        .get("started_serve")
        .and_then(Value::as_bool)
        .is_none()
    {
        push_incomplete_boundary_reason(&mut reasons);
    }
    if value.get("ran_product_smoke").and_then(Value::as_bool) == Some(true) {
        reasons.push("first-run evidence says product smoke ran");
    } else if value
        .get("ran_product_smoke")
        .and_then(Value::as_bool)
        .is_none()
    {
        push_incomplete_boundary_reason(&mut reasons);
    }
    if value.get("local_only").and_then(Value::as_bool) == Some(false) {
        reasons.push("first-run evidence is not local-only");
    } else if value.get("local_only").and_then(Value::as_bool).is_none() {
        push_incomplete_boundary_reason(&mut reasons);
    }
    match value
        .get("self_revision_write_path")
        .and_then(Value::as_str)
    {
        Some("run_reflection") => {}
        Some(_) => reasons.push("durable write path differs from run_reflection"),
        None => push_incomplete_boundary_reason(&mut reasons),
    }
    if value.get("daemon_enabled").and_then(Value::as_bool) == Some(true) {
        reasons.push("daemon is enabled");
    } else if value
        .get("daemon_enabled")
        .and_then(Value::as_bool)
        .is_none()
    {
        push_incomplete_boundary_reason(&mut reasons);
    }
    if value.get("daemon_writes_allowed").and_then(Value::as_bool) == Some(true) {
        reasons.push("daemon writes are allowed");
    } else if value
        .get("daemon_writes_allowed")
        .and_then(Value::as_bool)
        .is_none()
    {
        push_incomplete_boundary_reason(&mut reasons);
    }

    if reasons.is_empty() {
        gate(
            "local_read_only_boundary",
            "satisfied",
            Some(relative),
            "summary uses existing local evidence only and preserves run_reflection as the only durable write path",
        )
    } else {
        gate(
            "local_read_only_boundary",
            if reasons.contains(&FIRST_RUN_BOUNDARY_INCOMPLETE) {
                "not_verified"
            } else {
                "open"
            },
            Some(relative),
            reasons.join("; "),
        )
    }
}

fn push_incomplete_boundary_reason(reasons: &mut Vec<&'static str>) {
    if !reasons.contains(&FIRST_RUN_BOUNDARY_INCOMPLETE) {
        reasons.push(FIRST_RUN_BOUNDARY_INCOMPLETE);
    }
}

fn missing_paths(root: &Path, paths: &[&str]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| {
            root.join(path)
                .metadata()
                .map(|metadata| !metadata.is_file() || metadata.len() == 0)
                .unwrap_or(true)
        })
        .map(|path| (*path).to_string())
        .collect()
}

fn first_existing_file(root: &Path, paths: &[&'static str]) -> Option<(&'static str, PathBuf)> {
    paths
        .iter()
        .copied()
        .map(|relative| (relative, root.join(relative)))
        .find(|(_, path)| path.is_file())
}

fn first_existing_dir(root: &Path, paths: &[&'static str]) -> Option<(&'static str, PathBuf)> {
    paths
        .iter()
        .copied()
        .map(|relative| (relative, root.join(relative)))
        .find(|(_, path)| path.is_dir())
}

fn validate_support_bundle(path: &Path) -> Vec<String> {
    let mut reasons = Vec::new();
    if !support_bundle_manifest_is_valid(&path.join("manifest.json")) {
        reasons.push("support bundle manifest is not valid local alpha evidence".to_string());
    }
    if !support_bundle_doctor_is_valid(&path.join("doctor.json")) {
        reasons.push("support bundle doctor summary is not valid local alpha evidence".to_string());
    }
    if !support_bundle_product_smoke_summary_is_valid(&path.join("product-smoke-summary.json")) {
        reasons.push(
            "support bundle product smoke summary is not valid local alpha evidence".to_string(),
        );
    }
    reasons
}

fn support_bundle_manifest_is_valid(path: &Path) -> bool {
    let Ok(value) = read_json(path) else {
        return false;
    };
    value.get("bundle_format").and_then(Value::as_str)
        == Some("agent-llm-mm-local-alpha-support-bundle-v1")
        && value.get("local_only").and_then(Value::as_bool) == Some(true)
        && value.get("upload_performed").and_then(Value::as_bool) == Some(false)
}

fn support_bundle_doctor_is_valid(path: &Path) -> bool {
    let Ok(value) = read_json(path) else {
        return false;
    };
    value.get("status").and_then(Value::as_str) == Some("config-shape-ok")
        && value
            .get("self_revision_write_path")
            .and_then(Value::as_str)
            == Some("run_reflection")
        && value
            .get("runtime_bootstrap_performed")
            .and_then(Value::as_bool)
            == Some(false)
}

fn support_bundle_product_smoke_summary_is_valid(path: &Path) -> bool {
    let Ok(value) = read_json(path) else {
        return false;
    };
    value
        .get("required_artifacts")
        .and_then(Value::as_array)
        .is_some_and(|artifacts| artifacts.len() == PRODUCT_SMOKE_LATEST.len())
        && value
            .get("self_revision_write_path_expected")
            .and_then(Value::as_str)
            == Some("run_reflection")
}

fn overall_status(gates: &[LocalAlphaGateSummary]) -> &'static str {
    if gates.iter().all(|gate| gate.status == "satisfied") {
        "ready_for_human_review"
    } else if gates.iter().any(|gate| gate.status == "open") {
        "in_progress"
    } else {
        "not_verified"
    }
}

fn render_markdown(overall_status: &str, gates: &[LocalAlphaGateSummary]) -> String {
    let mut output = String::new();
    output.push_str("# Local Alpha Evidence Summary\n\n");
    output.push_str(&format!("Overall status: `{overall_status}`\n\n"));
    output.push_str(
        "Local Alpha is not complete unless every gate is `satisfied` with fresh, reviewable evidence, and a human release decision is still required. This summary is a read-only gate status rollup, not automatic certification. `run_reflection` remains the only durable identity/commitment/reflection write path.\n\n",
    );
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
    evidence_path: Option<&str>,
    reason: impl Into<String>,
) -> LocalAlphaGateSummary {
    LocalAlphaGateSummary {
        name,
        status,
        evidence_path: evidence_path.map(ToOwned::to_owned),
        reason: reason.into(),
    }
}

fn read_json(path: &Path) -> Result<Value> {
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
        serde_json::to_vec_pretty(value).context("serialize local alpha evidence summary")?,
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
