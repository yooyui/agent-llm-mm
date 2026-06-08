use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;

use crate::support::{
    packaging_archive::{EXPECTED_PACKAGING_ARCHIVES, validate_packaging_archive_manifest},
    release_candidate::validate_release_candidate,
};

#[derive(Debug, Clone)]
pub struct PackagingPreflightOptions {
    pub evidence_root: PathBuf,
    pub release_candidate: String,
    pub output_json_path: Option<PathBuf>,
    pub output_markdown_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackagingPreflightSummary {
    pub generated_at: String,
    pub kind: &'static str,
    pub release_candidate: String,
    pub packaging_ready: bool,
    pub overall_status: &'static str,
    pub source_only_release_artifacts: Vec<String>,
    pub blockers: Vec<PackagingPreflightBlocker>,
    pub non_claims: Vec<String>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackagingPreflightBlocker {
    pub subject: &'static str,
    pub status: &'static str,
    pub evidence_path: Option<String>,
    pub reason: String,
}

pub fn summarize_packaging_preflight(
    options: PackagingPreflightOptions,
) -> Result<PackagingPreflightSummary> {
    validate_release_candidate(&options.release_candidate)?;

    let source_only_release_artifacts =
        source_only_release_artifacts(&options.evidence_root, &options.release_candidate);
    let blockers = packaging_blockers(&options.evidence_root, &options.release_candidate);
    let packaging_ready = blockers.iter().all(|blocker| blocker.status == "satisfied");
    let overall_status = if packaging_ready { "ready" } else { "blocked" };
    let generated_at = Utc::now().to_rfc3339();
    let non_claims = vec![
        "not installer evidence".to_string(),
        "not service manager evidence".to_string(),
        "not auto-updater evidence".to_string(),
        "not uploaded release artifact evidence".to_string(),
        "not GA or production-ready packaging evidence".to_string(),
    ];
    let markdown = render_markdown(
        overall_status,
        &options.release_candidate,
        &source_only_release_artifacts,
        &blockers,
    );

    let summary = PackagingPreflightSummary {
        generated_at,
        kind: "packaging_preflight_summary",
        release_candidate: options.release_candidate,
        packaging_ready,
        overall_status,
        source_only_release_artifacts,
        blockers,
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

fn source_only_release_artifacts(root: &Path, release_candidate: &str) -> Vec<String> {
    let release_dir = root
        .join("target")
        .join("reports")
        .join("releases")
        .join(release_candidate);
    let mut artifacts = [
        "release-soak-summary.md",
        "compatibility-matrix.json",
        "release-boundaries.json",
        "release-decision.json",
    ]
    .into_iter()
    .filter(|file| release_dir.join(file).is_file())
    .map(str::to_string)
    .collect::<Vec<_>>();
    artifacts.sort();
    artifacts
}

fn packaging_blockers(root: &Path, release_candidate: &str) -> Vec<PackagingPreflightBlocker> {
    vec![
        binary_archive_blocker(root, release_candidate),
        blocker(
            "installer",
            "not_implemented",
            None,
            "installer generation and install verification are not implemented",
        ),
        blocker(
            "service_manager",
            "not_implemented",
            None,
            "service manager integration evidence is not implemented",
        ),
        blocker(
            "auto_updater",
            "not_implemented",
            None,
            "auto-updater packaging evidence is not implemented",
        ),
    ]
}

fn binary_archive_blocker(root: &Path, release_candidate: &str) -> PackagingPreflightBlocker {
    let relative_dir = format!("target/reports/releases/{release_candidate}/packaging");
    let packaging_dir = root.join(&relative_dir);
    let mut missing = Vec::new();
    let mut zero_byte = Vec::new();

    for name in EXPECTED_PACKAGING_ARCHIVES.iter().copied() {
        let path = packaging_dir.join(name);
        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() && metadata.len() > 0 => {}
            Ok(metadata) if metadata.is_file() => zero_byte.push(name),
            _ => missing.push(name),
        }
    }

    if missing.is_empty() && zero_byte.is_empty() {
        let manifest_validation = validate_packaging_archive_manifest(root, release_candidate);
        if manifest_validation.status != "satisfied" {
            return blocker(
                "binary_archive",
                manifest_validation.status,
                Some(relative_dir),
                manifest_validation.reason,
            );
        }

        return blocker(
            "binary_archive",
            "satisfied",
            Some(relative_dir),
            manifest_validation.reason,
        );
    }

    if !zero_byte.is_empty() {
        let mut reason = format!(
            "zero-byte binary archive placeholders are invalid: {}",
            zero_byte.join(", ")
        );
        if !missing.is_empty() {
            reason.push_str(&format!("; missing archives: {}", missing.join(", ")));
        }
        return blocker("binary_archive", "invalid", Some(relative_dir), reason);
    }

    if missing.len() == EXPECTED_PACKAGING_ARCHIVES.len() {
        blocker(
            "binary_archive",
            "missing",
            Some(relative_dir),
            "binary archive evidence is missing; source-only soak artifacts are not packaging evidence",
        )
    } else {
        blocker(
            "binary_archive",
            "incomplete",
            Some(relative_dir),
            format!("missing archives: {}", missing.join(", ")),
        )
    }
}

fn render_markdown(
    overall_status: &str,
    release_candidate: &str,
    source_only_release_artifacts: &[String],
    blockers: &[PackagingPreflightBlocker],
) -> String {
    let mut output = String::new();
    output.push_str("# Packaging Preflight Summary\n\n");
    output.push_str(&format!("- release_candidate: `{release_candidate}`\n"));
    output.push_str(&format!("- overall_status: `{overall_status}`\n"));
    output.push_str(&format!(
        "- source_only_release_artifacts: `{}`\n\n",
        if source_only_release_artifacts.is_empty() {
            "none".to_string()
        } else {
            source_only_release_artifacts.join(", ")
        }
    ));
    output.push_str("| Subject | Status | Evidence | Reason |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for blocker in blockers {
        output.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            blocker.subject,
            blocker.status,
            blocker.evidence_path.as_deref().unwrap_or(""),
            blocker.reason.replace('|', "\\|")
        ));
    }
    output
}

fn blocker(
    subject: &'static str,
    status: &'static str,
    evidence_path: Option<String>,
    reason: impl Into<String>,
) -> PackagingPreflightBlocker {
    PackagingPreflightBlocker {
        subject,
        status,
        evidence_path,
        reason: reason.into(),
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).context("serialize packaging preflight summary")?,
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
