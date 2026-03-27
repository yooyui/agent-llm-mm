use serde::Serialize;

use crate::{
    interfaces,
    support::config::{AppConfig, TransportKind},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub transport: TransportKind,
    pub database_url: String,
    pub status: &'static str,
}

pub async fn run_doctor(config: AppConfig) -> anyhow::Result<DoctorReport> {
    match config.transport {
        TransportKind::Stdio => interfaces::mcp::validate_stdio_runtime(&config).await?,
    }

    Ok(DoctorReport {
        transport: config.transport,
        database_url: config.database_url,
        status: "ok",
    })
}
