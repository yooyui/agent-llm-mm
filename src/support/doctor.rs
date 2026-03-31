use serde::Serialize;

use crate::{
    interfaces,
    support::config::{AppConfig, ModelProviderKind, TransportKind},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub transport: TransportKind,
    pub database_url: String,
    pub provider: ModelProviderKind,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub status: &'static str,
}

pub async fn run_doctor(config: AppConfig) -> anyhow::Result<DoctorReport> {
    config.validate_model_config().map_err(anyhow::Error::msg)?;

    let base_url = config.doctor_base_url();
    let model = config.doctor_model();

    match config.transport {
        TransportKind::Stdio => interfaces::mcp::validate_stdio_runtime(&config).await?,
    }

    Ok(DoctorReport {
        transport: config.transport,
        database_url: config.database_url,
        provider: config.model_provider,
        base_url,
        model,
        status: "ok",
    })
}
