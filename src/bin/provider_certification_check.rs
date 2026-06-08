use std::path::PathBuf;

use agent_llm_mm::support::{
    config::AppConfig,
    provider_certification::{ProviderCertificationOptions, summarize_provider_certification},
};
use anyhow::{Result, anyhow};

fn main() -> Result<()> {
    let mut config_path = None;
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
            "--config-path" => config_path = Some(next_path_value(&mut args, "--config-path")?),
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

    let config = if let Some(path) = config_path {
        AppConfig::load_from_path(path).map_err(anyhow::Error::msg)?
    } else {
        AppConfig::load().map_err(anyhow::Error::msg)?
    };

    let summary = summarize_provider_certification(ProviderCertificationOptions {
        config,
        evidence_root: evidence_root.unwrap_or_else(|| PathBuf::from(".")),
        output_json_path,
        output_markdown_path,
    })?;

    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: provider_certification_check [--config-path <path>] [--evidence-root <path>] [--output-json <path>] [--output-md <path>]"
    );
}

fn next_path_value(args: &mut impl Iterator<Item = String>, flag: &'static str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {flag}"))
}
