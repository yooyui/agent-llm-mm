use std::path::PathBuf;

use agent_llm_mm::support::{
    config::AppConfig,
    support_bundle::{SupportBundleOptions, generate_support_bundle},
};
use anyhow::{Result, anyhow};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let output_dir = match args.next() {
        Some(value) if value == "-h" || value == "--help" => {
            print_usage();
            return Ok(());
        }
        Some(value) => PathBuf::from(value),
        None => return Err(anyhow!("missing output_dir")),
    };
    let config_path = args.next().map(PathBuf::from);
    if args.next().is_some() {
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
    })
    .await?;

    println!("support bundle artifacts: {}", output_dir.display());
    Ok(())
}

fn print_usage() {
    eprintln!("usage: generate_support_bundle <output_dir> [config_path]");
}
