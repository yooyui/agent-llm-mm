use std::fs;

use agent_llm_mm::support::{
    packaging_archive::{
        PackagingArchiveOptions, generate_packaging_archive_evidence,
        validate_packaging_archive_manifest,
    },
    packaging_preflight::{PackagingPreflightOptions, summarize_packaging_preflight},
};
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tar::{Builder, Header};
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
fn archive_evidence_rejects_plain_text_archive_placeholders() {
    let temp_dir = tempdir().expect("temp dir");
    write_plain_text_archive_placeholders(temp_dir.path());

    let error = generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
    })
    .expect_err("plain text archive placeholders should be rejected");

    assert!(error.to_string().contains("invalid archive format"));
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
fn direct_manifest_validation_rejects_unsafe_candidate_names() {
    let temp_dir = tempdir().expect("temp dir");

    let validation = validate_packaging_archive_manifest(temp_dir.path(), "../escape");

    assert_eq!(validation.status, "invalid");
    assert!(validation.reason.contains("release candidate"));
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
fn packaging_preflight_rejects_plain_text_archives_even_with_matching_manifest() {
    let temp_dir = tempdir().expect("temp dir");
    write_plain_text_archive_placeholders(temp_dir.path());
    write_matching_manifest_for_existing_archives(temp_dir.path());

    let summary = summarize_packaging_preflight(PackagingPreflightOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("preflight should summarize placeholder archive files");

    assert_blocker(&summary, "binary_archive", "invalid");
    assert!(!summary.packaging_ready);
    let blocker = summary
        .blockers
        .iter()
        .find(|blocker| blocker.subject == "binary_archive")
        .expect("binary archive blocker");
    assert!(
        blocker.reason.contains("invalid archive format"),
        "reason should reject placeholder archives before checksum satisfaction: {}",
        blocker.reason
    );
}

#[test]
fn packaging_preflight_rejects_truncated_archives_even_with_matching_manifest() {
    let temp_dir = tempdir().expect("temp dir");
    write_truncated_archive_placeholders(temp_dir.path());
    write_matching_manifest_for_existing_archives(temp_dir.path());

    let summary = summarize_packaging_preflight(PackagingPreflightOptions {
        evidence_root: temp_dir.path().to_path_buf(),
        release_candidate: CANDIDATE.to_string(),
        output_json_path: None,
        output_markdown_path: None,
    })
    .expect("preflight should summarize truncated archive files");

    assert_blocker(&summary, "binary_archive", "invalid");
    assert!(!summary.packaging_ready);
    let blocker = summary
        .blockers
        .iter()
        .find(|blocker| blocker.subject == "binary_archive")
        .expect("binary archive blocker");
    assert!(
        blocker.reason.contains("invalid archive format"),
        "reason should reject truncated archives before checksum satisfaction: {}",
        blocker.reason
    );
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
        let archive_path = packaging_dir.join(archive);
        if archive.ends_with(".zip") {
            write_valid_zip_archive(&archive_path, archive);
        } else {
            write_valid_tar_gz_archive(&archive_path, index, archive);
        }
    }
}

fn write_valid_tar_gz_archive(path: &std::path::Path, index: usize, archive: &str) {
    let file = fs::File::create(path).expect("create tar.gz archive fixture");
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    let content = format!("archive fixture {index}: {archive}\n");
    let mut header = Header::new_gnu();
    header
        .set_path("agent-llm-mm/archive-fixture.txt")
        .expect("set tar path");
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append(&header, content.as_bytes())
        .expect("append tar fixture");
    let encoder = builder.into_inner().expect("finish tar archive");
    encoder.finish().expect("finish gzip archive");
}

fn write_valid_zip_archive(path: &std::path::Path, archive: &str) {
    let file_name = format!("{archive}.txt");
    let file_name_bytes = file_name.as_bytes();
    let local_header_len = 30 + file_name_bytes.len();
    let central_directory_len = 46 + file_name_bytes.len();
    let mut bytes = Vec::new();

    push_u32(&mut bytes, 0x0403_4b50);
    push_u16(&mut bytes, 20);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u16(&mut bytes, file_name_bytes.len() as u16);
    push_u16(&mut bytes, 0);
    bytes.extend_from_slice(file_name_bytes);

    push_u32(&mut bytes, 0x0201_4b50);
    push_u16(&mut bytes, 20);
    push_u16(&mut bytes, 20);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u16(&mut bytes, file_name_bytes.len() as u16);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    push_u32(&mut bytes, 0);
    bytes.extend_from_slice(file_name_bytes);

    push_u32(&mut bytes, 0x0605_4b50);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 1);
    push_u32(&mut bytes, central_directory_len as u32);
    push_u32(&mut bytes, local_header_len as u32);
    push_u16(&mut bytes, 0);

    fs::write(path, bytes).expect("write zip archive fixture");
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_plain_text_archive_placeholders(root: &std::path::Path) {
    let packaging_dir = packaging_dir(root);
    fs::create_dir_all(&packaging_dir).expect("create packaging dir");
    for (index, archive) in ARCHIVES.iter().enumerate() {
        fs::write(
            packaging_dir.join(archive),
            format!("plain text placeholder {index}: {archive}\n"),
        )
        .expect("write archive placeholder");
    }
}

fn write_truncated_archive_placeholders(root: &std::path::Path) {
    let packaging_dir = packaging_dir(root);
    fs::create_dir_all(&packaging_dir).expect("create packaging dir");
    for archive in ARCHIVES {
        let bytes = if archive.ends_with(".zip") {
            b"PK\x03\x04".to_vec()
        } else {
            vec![0x1f, 0x8b, 0x08, 0x00]
        };
        fs::write(packaging_dir.join(archive), bytes).expect("write truncated archive placeholder");
    }
}

fn write_matching_manifest_for_existing_archives(root: &std::path::Path) {
    let packaging_dir = packaging_dir(root);
    let archives = ARCHIVES
        .iter()
        .map(|archive| {
            let bytes = fs::read(packaging_dir.join(archive)).expect("read archive");
            json!({
                "name": archive,
                "size_bytes": bytes.len(),
                "sha256": format!("{:x}", Sha256::digest(&bytes))
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        manifest_path(root),
        serde_json::to_vec_pretty(&json!({
            "kind": "packaging_archive_evidence",
            "release_candidate": CANDIDATE,
            "generated_at": "2026-06-08T00:00:00Z",
            "local_only": true,
            "archives": archives,
            "complete": true,
            "non_claims": []
        }))
        .expect("manifest json"),
    )
    .expect("write matching manifest");
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
