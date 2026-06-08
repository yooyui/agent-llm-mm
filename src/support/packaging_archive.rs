use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use zip::ZipArchive;

use crate::support::release_candidate::validate_release_candidate;

pub const PACKAGING_ARCHIVE_MANIFEST: &str = "packaging-archive-manifest.json";
pub const EXPECTED_PACKAGING_ARCHIVES: [&str; 4] = [
    "agent-llm-mm-macos-aarch64.tar.gz",
    "agent-llm-mm-macos-x86_64.tar.gz",
    "agent-llm-mm-linux-x86_64.tar.gz",
    "agent-llm-mm-windows-x86_64.zip",
];

#[derive(Debug, Clone)]
pub struct PackagingArchiveOptions {
    pub evidence_root: PathBuf,
    pub release_candidate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackagingArchiveManifest {
    pub kind: String,
    pub release_candidate: String,
    pub generated_at: String,
    pub local_only: bool,
    pub archives: Vec<PackagingArchiveEntry>,
    pub complete: bool,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackagingArchiveEntry {
    pub name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagingArchiveManifestValidation {
    pub status: &'static str,
    pub reason: String,
}

pub fn generate_packaging_archive_evidence(
    options: PackagingArchiveOptions,
) -> Result<PackagingArchiveManifest> {
    validate_release_candidate(&options.release_candidate).context("validate release candidate")?;

    let packaging_dir = packaging_dir(&options.evidence_root, &options.release_candidate);
    let mut missing = Vec::new();
    let mut zero_byte = Vec::new();
    let mut archives = Vec::new();

    for name in EXPECTED_PACKAGING_ARCHIVES {
        let archive_path = packaging_dir.join(name);
        match fs::metadata(&archive_path) {
            Ok(metadata) if metadata.is_file() && metadata.len() > 0 => {
                validate_archive_file_format(&archive_path, name)
                    .with_context(|| format!("invalid archive format for {name}"))?;
                archives.push(PackagingArchiveEntry {
                    name: name.to_string(),
                    size_bytes: metadata.len(),
                    sha256: sha256_file(&archive_path)
                        .with_context(|| format!("hash archive {name}"))?,
                });
            }
            Ok(metadata) if metadata.is_file() => zero_byte.push(name),
            _ => missing.push(name),
        }
    }

    if !zero_byte.is_empty() {
        return Err(anyhow!(
            "zero-byte archives are invalid: {}",
            zero_byte.join(", ")
        ));
    }
    if !missing.is_empty() {
        return Err(anyhow!("missing archives: {}", missing.join(", ")));
    }

    let manifest = PackagingArchiveManifest {
        kind: "packaging_archive_evidence".to_string(),
        release_candidate: options.release_candidate.clone(),
        generated_at: Utc::now().to_rfc3339(),
        local_only: true,
        archives,
        complete: true,
        non_claims: vec![
            "not evidence that this command built compiled binaries".to_string(),
            "not installer evidence".to_string(),
            "not service manager evidence".to_string(),
            "not auto-updater evidence".to_string(),
            "not upload, tag, GA, or production-ready packaging evidence".to_string(),
        ],
    };

    fs::create_dir_all(&packaging_dir)
        .with_context(|| format!("create packaging evidence dir {}", packaging_dir.display()))?;
    fs::write(
        manifest_path(&options.evidence_root, &options.release_candidate),
        serde_json::to_vec_pretty(&manifest).context("serialize packaging archive manifest")?,
    )
    .with_context(|| {
        format!(
            "write {}",
            manifest_path(&options.evidence_root, &options.release_candidate).display()
        )
    })?;

    Ok(manifest)
}

pub fn validate_packaging_archive_manifest(
    root: &Path,
    release_candidate: &str,
) -> PackagingArchiveManifestValidation {
    if let Err(error) = validate_release_candidate(release_candidate) {
        return validation(
            "invalid",
            format!("invalid release candidate for packaging archive manifest: {error}"),
        );
    }

    let manifest_path = manifest_path(root, release_candidate);
    let manifest_bytes = match fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return validation(
                "incomplete",
                "packaging archive manifest is missing; run archive evidence generation after archives exist",
            );
        }
    };
    let manifest = match serde_json::from_slice::<PackagingArchiveManifest>(&manifest_bytes) {
        Ok(manifest) => manifest,
        Err(error) => {
            return validation(
                "invalid",
                format!("packaging archive manifest is not valid JSON: {error}"),
            );
        }
    };

    if manifest.kind != "packaging_archive_evidence" {
        return validation(
            "invalid",
            format!(
                "packaging archive manifest has unexpected kind {}",
                manifest.kind
            ),
        );
    }
    if manifest.release_candidate != release_candidate {
        return validation(
            "invalid",
            format!(
                "packaging archive manifest release candidate {} does not match {}",
                manifest.release_candidate, release_candidate
            ),
        );
    }
    if !manifest.local_only {
        return validation(
            "invalid",
            "packaging archive manifest must be marked local_only=true",
        );
    }
    if !manifest.complete {
        return validation(
            "incomplete",
            "packaging archive manifest is marked complete=false",
        );
    }

    let archive_names = manifest
        .archives
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();
    if manifest.archives.len() != EXPECTED_PACKAGING_ARCHIVES.len() {
        return validation(
            "invalid",
            format!(
                "packaging archive manifest must contain exactly {} archive entries",
                EXPECTED_PACKAGING_ARCHIVES.len()
            ),
        );
    }

    let missing_entries = EXPECTED_PACKAGING_ARCHIVES
        .iter()
        .copied()
        .filter(|expected| !archive_names.contains(expected))
        .collect::<Vec<_>>();
    if !missing_entries.is_empty() {
        return validation(
            "incomplete",
            format!(
                "packaging archive manifest is missing archive entries: {}",
                missing_entries.join(", ")
            ),
        );
    }

    let unexpected_entries = archive_names
        .iter()
        .copied()
        .filter(|name| !EXPECTED_PACKAGING_ARCHIVES.contains(name))
        .collect::<Vec<_>>();
    if !unexpected_entries.is_empty() {
        return validation(
            "invalid",
            format!(
                "packaging archive manifest has unexpected archive entries: {}",
                unexpected_entries.join(", ")
            ),
        );
    }

    for expected in EXPECTED_PACKAGING_ARCHIVES {
        let entry = manifest
            .archives
            .iter()
            .find(|entry| entry.name == expected)
            .expect("entry presence checked above");
        let archive_path = packaging_dir(root, release_candidate).join(expected);
        let metadata = match fs::metadata(&archive_path) {
            Ok(metadata) if metadata.is_file() => metadata,
            _ => {
                return validation(
                    "incomplete",
                    format!("manifest archive file is missing: {expected}"),
                );
            }
        };
        if metadata.len() != entry.size_bytes {
            return validation(
                "invalid",
                format!(
                    "manifest size mismatch for {expected}: manifest={}, actual={}",
                    entry.size_bytes,
                    metadata.len()
                ),
            );
        }
        if let Err(error) = validate_archive_file_format(&archive_path, expected) {
            return validation("invalid", error.to_string());
        }

        let actual_sha256 = match sha256_file(&archive_path) {
            Ok(sha256) => sha256,
            Err(error) => {
                return validation(
                    "invalid",
                    format!("failed to hash archive {expected}: {error}"),
                );
            }
        };
        if actual_sha256 != entry.sha256 {
            return validation(
                "invalid",
                format!("manifest sha256 mismatch for {expected}"),
            );
        }
    }

    validation(
        "satisfied",
        "all expected archive files match the packaging archive manifest",
    )
}

pub fn packaging_dir(root: &Path, release_candidate: &str) -> PathBuf {
    root.join("target")
        .join("reports")
        .join("releases")
        .join(release_candidate)
        .join("packaging")
}

pub fn manifest_path(root: &Path, release_candidate: &str) -> PathBuf {
    packaging_dir(root, release_candidate).join(PACKAGING_ARCHIVE_MANIFEST)
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_archive_file_format(path: &Path, name: &str) -> Result<()> {
    if name.ends_with(".tar.gz") {
        return validate_tar_gz_archive(path, name).with_context(|| {
            format!("invalid archive format for {name}: expected gzip-compressed tar archive")
        });
    }
    if name.ends_with(".zip") {
        return validate_zip_archive(path, name)
            .with_context(|| format!("invalid archive format for {name}: expected zip archive"));
    }

    Err(anyhow!(
        "invalid archive format for {name}: expected gzip-compressed tar archive or zip archive"
    ))
}

fn validate_tar_gz_archive(path: &Path, name: &str) -> Result<()> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut decoder = GzDecoder::new(file);
    let mut has_entries = false;

    {
        let mut archive = tar::Archive::new(&mut decoder);
        let entries = archive
            .entries()
            .with_context(|| format!("read tar entries for {name}"))?;
        for entry in entries {
            let mut entry = entry.with_context(|| format!("read tar entry for {name}"))?;
            io::copy(&mut entry, &mut io::sink())
                .with_context(|| format!("read tar entry contents for {name}"))?;
            has_entries = true;
        }
    }

    if !has_entries {
        return Err(anyhow!(
            "invalid archive format for {name}: gzip-compressed tar archive contains no entries"
        ));
    }

    io::copy(&mut decoder, &mut io::sink())
        .with_context(|| format!("finish gzip stream for {name}"))?;
    Ok(())
}

fn validate_zip_archive(path: &Path, name: &str) -> Result<()> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut archive =
        ZipArchive::new(file).with_context(|| format!("read zip central directory for {name}"))?;

    if archive.is_empty() {
        return Err(anyhow!(
            "invalid archive format for {name}: zip archive contains no entries"
        ));
    }

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index_raw(index)
            .with_context(|| format!("read raw zip entry {index} for {name}"))?;
        io::copy(&mut entry, &mut io::sink())
            .with_context(|| format!("read raw zip entry contents for {name}"))?;
    }

    Ok(())
}

fn validation(
    status: &'static str,
    reason: impl Into<String>,
) -> PackagingArchiveManifestValidation {
    PackagingArchiveManifestValidation {
        status,
        reason: reason.into(),
    }
}
