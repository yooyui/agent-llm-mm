use agent_llm_mm::{
    startup_transport_from_default_config,
    support::config::{AppConfig, TransportKind},
};

#[test]
fn default_config_uses_stdio_transport() {
    let config = AppConfig::default();
    assert_eq!(config.transport, TransportKind::Stdio);
}

#[test]
fn startup_transport_uses_default_config_stdio() {
    assert_eq!(startup_transport_from_default_config(), TransportKind::Stdio);
}
