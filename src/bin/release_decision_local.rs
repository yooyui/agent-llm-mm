use std::path::PathBuf;

use agent_llm_mm::support::release_decision::{
    ReleaseDecisionOptions, ReleaseDecisionRequest, write_release_decision,
};
use anyhow::{Result, anyhow};

fn main() -> Result<()> {
    let mut release_candidate = None;
    let mut evidence_root = None;
    let mut decision = "blocked".to_string();
    let mut human_reviewer = None;
    let mut rollback_note = None;
    let mut output_json_path = None;
    let mut output_markdown_path = None;

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
            "--decision" => decision = next_string_value(&mut args, "--decision")?,
            "--human-reviewer" => {
                human_reviewer = Some(next_string_value(&mut args, "--human-reviewer")?)
            }
            "--rollback-note" => {
                rollback_note = Some(next_string_value(&mut args, "--rollback-note")?)
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

    let release_candidate =
        release_candidate.ok_or_else(|| anyhow!("missing required --release-candidate <name>"))?;
    let artifact = write_release_decision(ReleaseDecisionOptions {
        evidence_root: evidence_root.unwrap_or_else(|| PathBuf::from(".")),
        release_candidate,
        request: ReleaseDecisionRequest {
            decision,
            human_reviewer,
            rollback_note,
        },
        output_json_path,
        output_markdown_path,
    })?;

    println!("{}", serde_json::to_string_pretty(&artifact)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: release_decision_local --release-candidate <name> [--decision blocked|rejected|deferred|approved] [--human-reviewer <name>] [--rollback-note <text>] [--evidence-root <path>] [--output-json <path>] [--output-md <path>]"
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
