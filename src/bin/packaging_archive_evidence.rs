use std::path::PathBuf;

use agent_llm_mm::support::packaging_archive::{
    PackagingArchiveOptions, generate_packaging_archive_evidence,
};
use anyhow::{Result, anyhow};

fn main() -> Result<()> {
    let mut release_candidate = None;
    let mut evidence_root = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "--release-candidate" => {
                release_candidate = Some(next_string_value(&mut args, "--release-candidate")?)
            }
            "--evidence-root" => {
                evidence_root = Some(next_path_value(&mut args, "--evidence-root")?)
            }
            _ => return Err(anyhow!("unknown argument: {arg}")),
        }
    }

    let release_candidate =
        release_candidate.ok_or_else(|| anyhow!("missing required --release-candidate <name>"))?;
    let manifest = generate_packaging_archive_evidence(PackagingArchiveOptions {
        evidence_root: evidence_root.unwrap_or_else(|| PathBuf::from(".")),
        release_candidate,
    })?;

    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: packaging_archive_evidence --release-candidate <name> [--evidence-root <path>]"
    );
}

fn next_path_value(args: &mut impl Iterator<Item = String>, flag: &'static str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {flag}"))
}

fn next_string_value(
    args: &mut impl Iterator<Item = String>,
    flag: &'static str,
) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value for {flag}"))
}
