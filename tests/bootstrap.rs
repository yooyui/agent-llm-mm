use agent_llm_mm::{
    run_command, run_doctor, startup_transport_from_default_config,
    support::{
        cli::{AppCommand, command_from_args},
        config::{AppConfig, ModelConfig, ModelProviderKind, TransportKind},
    },
};
use std::{time::Duration, vec};
use tempfile::tempdir;

#[test]
fn default_config_uses_stdio_transport() {
    let config = AppConfig::default();
    assert_eq!(config.transport, TransportKind::Stdio);
}

#[test]
fn default_config_uses_file_backed_sqlite_database() {
    let config = AppConfig::default();

    assert_ne!(config.database_url, "sqlite::memory:");
    assert!(config.database_url.starts_with("sqlite://"));
}

#[test]
fn startup_transport_uses_default_config_stdio() {
    assert_eq!(
        startup_transport_from_default_config(),
        TransportKind::Stdio
    );
}

#[test]
fn cli_defaults_to_serve_when_no_subcommand_is_provided() {
    let command = command_from_args(vec!["agent_llm_mm".to_string()]).expect("command");

    assert_eq!(command, AppCommand::Serve);
}

#[test]
fn cli_accepts_doctor_subcommand() {
    let command =
        command_from_args(vec!["agent_llm_mm".to_string(), "doctor".to_string()]).expect("command");

    assert_eq!(command, AppCommand::Doctor);
}

#[test]
fn cli_rejects_unknown_subcommand() {
    let error = command_from_args(vec!["agent_llm_mm".to_string(), "wat".to_string()])
        .expect_err("unknown command should fail");

    assert!(error.to_string().contains("unsupported command"));
}

#[tokio::test]
async fn doctor_bootstraps_configured_sqlite_database_and_returns_report() {
    let temp_dir = tempdir().expect("temp dir");
    let database_path = temp_dir.path().join("doctor.sqlite");
    let database_url = format!(
        "sqlite://{}",
        database_path.to_string_lossy().replace('\\', "/")
    );
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url: database_url.clone(),
        model_provider: ModelProviderKind::Mock,
        model_config: ModelConfig::Mock,
    };

    let report = run_doctor(config).await.expect("doctor should pass");

    assert_eq!(report.transport, TransportKind::Stdio);
    assert_eq!(report.database_url, database_url);
    assert_eq!(report.provider, ModelProviderKind::Mock);
    assert_eq!(report.base_url, None);
    assert_eq!(report.model, None);
    assert!(
        database_path.exists(),
        "doctor should create sqlite database"
    );
}

#[tokio::test]
async fn run_uses_stdio_server_path_and_does_not_exit_immediately() {
    let handle = tokio::spawn(agent_llm_mm::run());

    for _ in 0..10 {
        tokio::task::yield_now().await;
        if handle.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    assert!(
        !handle.is_finished(),
        "run() returned immediately instead of serving stdio"
    );

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn serve_command_uses_explicit_config_for_bootstrap() {
    let temp_dir = tempdir().expect("temp dir");
    let database_path = temp_dir.path().join("missing-parent").join("serve.sqlite");
    let database_url = format!(
        "sqlite://{}",
        database_path.to_string_lossy().replace('\\', "/")
    );
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url,
        model_provider: ModelProviderKind::Mock,
        model_config: ModelConfig::Mock,
    };

    let handle = tokio::spawn(run_command(AppCommand::Serve, config));

    for _ in 0..10 {
        tokio::task::yield_now().await;
        if handle.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    assert!(
        handle.is_finished(),
        "serve should use the explicit config and fail fast for an invalid database path"
    );

    let result = handle.await.expect("join");
    assert!(result.is_err(), "invalid config should return an error");
}
