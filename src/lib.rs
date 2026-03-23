use anyhow::Result;

pub mod application;
pub mod domain;
pub mod error;
pub mod ports;
pub mod support;

use support::config::{AppConfig, TransportKind};

pub fn startup_transport_from_default_config() -> TransportKind {
    AppConfig::default().transport
}

pub async fn run() -> Result<()> {
    support::tracing::init_tracing();

    match startup_transport_from_default_config() {
        TransportKind::Stdio => Ok(()),
    }
}
