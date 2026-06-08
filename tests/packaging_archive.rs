use std::fs;

use agent_llm_mm::support::{
    packaging_archive::{PackagingArchiveOptions, generate_packaging_archive_evidence},
    packaging_preflight::{PackagingPreflightOptions, summarize_packaging_preflight},
};
use serde_json::Value;
use tempfile::tempdir;

const CANDIDATE: &str = "local-alpha-20260608.1-rc.1";
const ARCHIVES: [&str; 4] = [
    "agent-llm-mm-macos-aarch64.tar.gz",
    "agent-llm-mm-macos-x86_64.tar.gz",
    "agent-llm-mm-linux-x86_64.tar.gz",
    "agent-llm-mm-windows-x86_64.zip",
];

#[test]
fn archive_evidence_rejects_missing_archives() {
    let temp_dir = tempdir().expect("temp dir");

    let error = generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
    })
    .expect_err("missing archives should be rejected");

    assert!(error.to_string().contains("missing archives"));
    assert!(
        !manifest_path(temp_dir.path()).exists(),
        "failed generation must not write manifest"
    );
}

#[test]
fn archive_evidence_rejects_zero_byte_archives() {
    let temp_dir = tempdir().expect("temp dir");
    let packaging_dir = packaging_dir(temp_dir.path());
    fs::create_dir_all(&packaging_dir).expect("create packaging dir");
    for archive in ARCHIVES {
        fs::write(packaging_dir.join(archive), []).expect("write zero-byte archive");
    }

    let error = generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
    })
    .expect_err("zero-byte archives should be rejected");

    assert!(error.to_string().contains("zero-byte archives"));
    assert!(
        !manifest_path(temp_dir.path()).exists(),
        "failed generation must not write manifest"
    );
}

#[test]
fn archive_evidence_writes_manifest_with_archive_checksums() {
    let temp_dir = tempdir().expect("temp dir");
    write_complete_archives(temp_dir.path());

    let manifest = generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
    })
    .expect("complete archive set should write manifest");

    assert_eq!(manifest.kind, "packaging_archive_evidence");
    assert_eq!(manifest.release_candidate, CANDIDATE);
    assert!(manifest.local_only);
    assert!(manifest.complete);
    assert_eq!(manifest.archives.len(), ARCHIVES.len());
    assert!(
        manifest
            .non_claims
            .iter()
            .any(|claim| claim.contains("not installer evidence"))
    );

    let raw_manifest =
        fs::read_to_string(manifest_path(temp_dir.path())).expect("read written manifest");
    assert!(
        !raw_manifest.contains(temp_dir.path().to_string_lossy().as_ref()),
        "manifest must not include absolute paths"
    );

    let manifest_json: Value = serde_json::from_str(&raw_manifest).expect("parse manifest json");
    assert_eq!(manifest_json["complete"], true);
    assert_eq!(manifest_json["archives"].as_array().unwrap().len(), 4);
    for archive in ARCHIVES {
        let entry = manifest
            .archives
            .iter()
            .find(|entry| entry.name == archive)
            .unwrap_or_else(|| panic!("missing manifest entry for {archive}"));
        assert!(entry.size_bytes > 0);
        assert_eq!(entry.sha256.len(), 64);
        assert!(entry.sha256.chars().all(|ch| ch.is_ascii_hexdigit()));
    }
}

#[test]
fn archive_evidence_rejects_unsafe_candidate_names() {
    let temp_dir = tempdir().expect("temp dir");

    let error = generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: "../escape".to_string(),
    })
    .expect_err("unsafe candidate name should be rejected");

    assert!(error.to_string().contains("release candidate"));
}

#[test]
fn packaging_preflight_rejects_manifest_archive_mismatch() {
    let temp_dir = tempdir().expect("temp dir");
    write_complete_archives(temp_dir.path());
    generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
    })
    .expect("write manifest");
    fs::write(
        packaging_dir(temp_dir.path()).join("agent-llm-mm-linux-x86_64.tar.gz"),
        b"changed after manifest",
    )
    .expect("mutate archive after manifest");

    let summary = summarize_packaging_preflight(PackagingPreflightOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("preflight should summarize mismatch");

    assert_blocker(&summary, "binary_archive", "invalid");
    assert!(!summary.packaging_ready);
}

#[test]
fn packaging_preflight_rejects_complete_archives_without_manifest() {
    let temp_dir = tempdir().expect("temp dir");
    write_complete_archives(temp_dir.path());

    let summary = summarize_packaging_preflight(PackagingPreflightOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("preflight should summarize missing manifest");

    assert_blocker(&summary, "binary_archive", "incomplete");
    assert!(!summary.packaging_ready);
}

#[test]
fn complete_archive_evidence_satisfies_only_binary_archive() {
    let temp_dir = tempdir().expect("temp dir");
    write_complete_archives(temp_dir.path());
    generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
    })
    .expect("write manifest");

    let summary = summarize_packaging_preflight(PackagingPreflightOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("preflight should summarize complete archive evidence");

    assert_blocker(&summary, "binary_archive", "satisfied");
    assert_blocker(&summary, "installer", "not_implemented");
    assert_blocker(&summary, "service_manager", "not_implemented");
    assert_blocker(&summary, "auto_updater", "not_implemented");
    assert!(!summary.packaging_ready);
    assert_eq!(summary.overall_status, "blocked");
}

fn write_complete_archives(root: &std::path::Path) {
    let packaging_dir = packaging_dir(root);
    fs::create_dir_all(&packaging_dir).expect("create packaging dir");
    for (index, archive) in ARCHIVES.iter().enumerate() {
        fs::write(
            packaging_dir.join(archive),
            format!("source-only archive fixture {index}: {archive}\n"),
        )
        .expect("write archive fixture");
    }
}

fn packaging_dir(root: &std::path::Path) -> std::path::PathBuf {
    root.join("target")
        .join("reports")
        .join("releases")
        .join(CANDIDATE)
        .join("packaging")
}

fn manifest_path(root: &std::path::Path) -> std::path::PathBuf {
    packaging_dir(root).join("packaging-archive-manifest.json")
}

fn assert_blocker(
    summary: &agent_llm_mm::support::packaging_preflight::PackagingPreflightSummary,
    subject: &str,
    status: &str,
) {
    let blocker = summary
        .blockers
        .iter()
        .find(|blocker| blocker.subject == subject)
        .unwrap_or_else(|| {
            panic!(
                "missing packaging blocker {subject}; blockers={:?}",
                summary.blockers
            )
        });
    assert_eq!(blocker.status, status);
}
