use std::path::PathBuf;

use agent_llm_mm::support::{
    config::AppConfig,
    provider_live_certification::{
        ProviderLiveCertificationMode, ProviderLiveCertificationOptions,
        run_provider_live_certification,
    },
};
use anyhow::{Result, anyhow};

fn main() -> Result<()> {
    let mut config_path = None;
    let mut evidence_root = None;
    let mut stub_evidence = false;

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
            "--stub-evidence" => stub_evidence = true,
            _ => return Err(anyhow!("unknown argument: {arg}")),
        }
    }

    let config = if let Some(path) = config_path {
        AppConfig::load_from_path(path).map_err(anyhow::Error::msg)?
    } else {
        AppConfig::load().map_err(anyhow::Error::msg)?
    };
    let mode = if stub_evidence {
        ProviderLiveCertificationMode::StubEvidence
    } else {
        ProviderLiveCertificationMode::Live
    };

    let report = run_provider_live_certification(ProviderLiveCertificationOptions {
        config,
        evidence_root: evidence_root.unwrap_or_else(|| PathBuf::from(".")),
        mode,
    })?;

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: provider_live_certification_run --stub-evidence [--config-path <path>] [--evidence-root <path>]\n\nWithout --stub-evidence, live provider certification is rejected until real network checks are implemented."
    );
}

fn next_path_value(args: &mut impl Iterator<Item = String>, flag: &'static str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {flag}"))
}
