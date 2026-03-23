use anyhow::Result;

pub mod adapters;
pub mod application;
pub mod domain;
pub mod error;
pub mod interfaces;
pub mod ports;
pub mod support;

use support::config::{AppConfig, TransportKind};

pub fn startup_transport_from_default_config() -> TransportKind {
    AppConfig::default().transport
}

pub async fn run() -> Result<()> {
    support::tracing::init_tracing();

    match startup_transport_from_default_config() {
        TransportKind::Stdio => interfaces::mcp::run_stdio_server().await,
    }
}
