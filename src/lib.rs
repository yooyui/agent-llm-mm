use anyhow::Result;
use serde::Serialize;

pub mod adapters;
pub mod application;
pub mod domain;
pub mod error;
pub mod interfaces;
pub mod ports;
pub mod support;

use support::{
    cli::AppCommand,
    config::{AppConfig, TransportKind},
    doctor::DoctorReport,
};

pub fn startup_transport_from_default_config() -> TransportKind {
    AppConfig::default().transport
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum RunOutput {
    Doctor(DoctorReport),
}

pub async fn run_command(command: AppCommand, config: AppConfig) -> Result<Option<RunOutput>> {
    match command {
        AppCommand::Serve => {
            match config.transport {
                TransportKind::Stdio => {
                    interfaces::mcp::run_stdio_server_with_config(config).await?
                }
            }

            Ok(None)
        }
        AppCommand::Doctor => run_doctor(config)
            .await
            .map(|report| Some(RunOutput::Doctor(report))),
    }
}

pub async fn run_doctor(config: AppConfig) -> Result<DoctorReport> {
    support::doctor::run_doctor(config).await
}

pub async fn run() -> Result<()> {
    support::tracing::init_tracing();
    let config = AppConfig::load().map_err(anyhow::Error::msg)?;
    run_command(AppCommand::Serve, config).await.map(|_| ())
}
