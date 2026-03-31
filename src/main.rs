use agent_llm_mm::{
    RunOutput, run_command,
    support::{cli::command_from_args, config::AppConfig},
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let command = command_from_args(std::env::args())?;
    let config = AppConfig::load().map_err(anyhow::Error::msg)?;
    match run_command(command, config).await? {
        Some(RunOutput::Doctor(report)) => {
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        None => Ok(()),
    }
}
