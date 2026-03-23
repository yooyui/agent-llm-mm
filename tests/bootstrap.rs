use agent_llm_mm::support::config::{AppConfig, TransportKind};

#[test]
fn default_config_uses_stdio_transport() {
    let config = AppConfig::default();
    assert_eq!(config.transport, TransportKind::Stdio);
}
