use std::path::PathBuf;

use agent_llm_mm::support::local_alpha_evidence::{
    LocalAlphaEvidenceOptions, summarize_local_alpha_evidence,
};
use anyhow::{Result, anyhow};

fn main() -> Result<()> {
    let mut evidence_root = None;
    let mut output_json_path = None;
    let mut output_markdown_path = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "--evidence-root" => {
                evidence_root = Some(next_path_value(&mut args, "--evidence-root")?)
            }
            "--output-json" => {
                output_json_path = Some(next_path_value(&mut args, "--output-json")?)
            }
            "--output-md" => {
                output_markdown_path = Some(next_path_value(&mut args, "--output-md")?)
            }
            _ => return Err(anyhow!("unknown argument: {arg}")),
        }
    }

    let summary = summarize_local_alpha_evidence(LocalAlphaEvidenceOptions {
        evidence_root: evidence_root.unwrap_or_else(|| PathBuf::from(".")),
        output_json_path,
        output_markdown_path,
    })?;

    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: local_alpha_evidence_summary [--evidence-root <path>] [--output-json <path>] [--output-md <path>]"
    );
}

fn next_path_value(args: &mut impl Iterator<Item = String>, flag: &'static str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {flag}"))
}
