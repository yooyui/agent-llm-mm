use std::path::PathBuf;

use agent_llm_mm::support::{
    config::AppConfig,
    support_bundle::{SupportBundleOptions, generate_support_bundle},
};
use anyhow::{Result, anyhow};

#[tokio::main]
async fn main() -> Result<()> {
    let mut local_log_path = None;
    let mut positional = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(value) = args.next() {
        if value == "-h" || value == "--help" {
            print_usage();
            return Ok(());
        }
        if value == "--log-file" {
            let Some(path) = args.next() else {
                return Err(anyhow!("missing value for --log-file"));
            };
            local_log_path = Some(PathBuf::from(path));
            continue;
        }
        if value.starts_with('-') {
            return Err(anyhow!("unknown option: {value}"));
        }
        positional.push(value);
    }

    let output_dir = match positional.first() {
        Some(value) => PathBuf::from(value),
        None => return Err(anyhow!("missing output_dir")),
    };
    let config_path = positional.get(1).map(PathBuf::from);
    if positional.len() > 2 {
        return Err(anyhow!("too many arguments"));
    }

    let config = if let Some(path) = &config_path {
        AppConfig::load_from_path(path).map_err(anyhow::Error::msg)?
    } else {
        AppConfig::load().map_err(anyhow::Error::msg)?
    };
    let project_root = std::env::current_dir()?;

    generate_support_bundle(SupportBundleOptions {
        config,
        output_dir: output_dir.clone(),
        config_path,
        project_root,
        local_log_path,
    })
    .await?;

    println!("support bundle artifacts: {}", output_dir.display());
    Ok(())
}

fn print_usage() {
    eprintln!("usage: generate_support_bundle <output_dir> [config_path] [--log-file <path>]");
}
