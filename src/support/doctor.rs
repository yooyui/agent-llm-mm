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
    pub auto_reflection_runtime_hooks: Vec<String>,
    pub self_revision_write_path: &'static str,
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
        auto_reflection_runtime_hooks: interfaces::mcp::server::AUTO_REFLECTION_RUNTIME_HOOKS
            .iter()
            .map(|hook| hook.to_string())
            .collect(),
        self_revision_write_path: interfaces::mcp::server::SELF_REVISION_WRITE_PATH,
        status: "ok",
    })
}
