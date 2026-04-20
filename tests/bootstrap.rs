use agent_llm_mm::{
    run_command, run_doctor, startup_transport_from_default_config,
    support::{
        cli::{AppCommand, command_from_args},
        config::{AppConfig, DATABASE_URL_ENV_VAR, ModelConfig, ModelProviderKind, TransportKind},
    },
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Mutex, OnceLock},
    time::Duration,
    vec,
};
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
#[cfg(target_os = "macos")]
fn default_config_uses_user_scoped_database_under_application_support() {
    let temp_dir = tempdir().expect("temp dir");
    let home_dir = temp_dir.path().join("home");
    let _guard = EnvGuard::apply([
        EnvChange::Unset(DATABASE_URL_ENV_VAR),
        EnvChange::Unset("XDG_DATA_HOME"),
        EnvChange::Set("HOME", home_dir.to_string_lossy().into_owned()),
    ]);

    let config = AppConfig::default();

    assert_eq!(
        config.database_url,
        sqlite_url(
            &home_dir
                .join("Library")
                .join("Application Support")
                .join("agent-llm-mm")
                .join("agent-llm-mm.sqlite")
        )
    );
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn default_config_uses_xdg_data_home_for_user_scoped_database() {
    let temp_dir = tempdir().expect("temp dir");
    let home_dir = temp_dir.path().join("home");
    let xdg_data_home = temp_dir.path().join("xdg-data");
    let _guard = EnvGuard::apply([
        EnvChange::Unset(DATABASE_URL_ENV_VAR),
        EnvChange::Set("HOME", home_dir.to_string_lossy().into_owned()),
        EnvChange::Set(
            "XDG_DATA_HOME",
            xdg_data_home.to_string_lossy().into_owned(),
        ),
    ]);

    let config = AppConfig::default();

    assert_eq!(
        config.database_url,
        sqlite_url(
            &xdg_data_home
                .join("agent-llm-mm")
                .join("agent-llm-mm.sqlite")
        )
    );
}

#[test]
#[cfg(windows)]
fn default_config_uses_local_app_data_for_user_scoped_database() {
    let temp_dir = tempdir().expect("temp dir");
    let local_app_data = temp_dir.path().join("AppData").join("Local");
    let _guard = EnvGuard::apply([
        EnvChange::Unset(DATABASE_URL_ENV_VAR),
        EnvChange::Set(
            "LOCALAPPDATA",
            local_app_data.to_string_lossy().into_owned(),
        ),
    ]);

    let config = AppConfig::default();

    assert_eq!(
        config.database_url,
        sqlite_url(
            &local_app_data
                .join("agent-llm-mm")
                .join("agent-llm-mm.sqlite")
        )
    );
}

#[test]
fn default_config_prefers_explicit_database_url_env_override() {
    let _guard = EnvGuard::apply([
        EnvChange::Set(
            DATABASE_URL_ENV_VAR,
            "sqlite:///tmp/agent-llm-mm-explicit.sqlite".to_string(),
        ),
        EnvChange::Unset("XDG_DATA_HOME"),
    ]);

    let config = AppConfig::default();

    assert_eq!(
        config.database_url,
        "sqlite:///tmp/agent-llm-mm-explicit.sqlite"
    );
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
async fn doctor_reports_self_revision_runtime_coverage() {
    let report = run_doctor(AppConfig::default())
        .await
        .expect("doctor should pass");

    assert_eq!(
        report.auto_reflection_runtime_hooks,
        vec![
            "ingest_interaction:failure".to_string(),
            "ingest_interaction:conflict".to_string(),
            "decide_with_snapshot:conflict".to_string(),
            "build_self_snapshot:periodic".to_string(),
        ]
    );
    assert_eq!(report.self_revision_write_path, "run_reflection");
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
async fn doctor_creates_missing_parent_directories_for_file_backed_sqlite_database() {
    let temp_dir = tempdir().expect("temp dir");
    let database_path = temp_dir.path().join("missing-parent").join("serve.sqlite");
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

    assert_eq!(report.database_url, database_url);
    assert!(
        database_path.exists(),
        "doctor should create missing parent directories for file-backed sqlite databases"
    );
}

#[tokio::test]
async fn serve_command_fails_fast_for_malformed_database_url() {
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url: "not-a-sqlite-url".to_string(),
        model_provider: ModelProviderKind::Mock,
        model_config: ModelConfig::Mock,
    };

    let result = run_command(AppCommand::Serve, config).await;

    assert!(
        result.is_err(),
        "malformed database url should return an error"
    );
}

fn sqlite_url(path: &Path) -> String {
    format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
}

enum EnvChange {
    Set(&'static str, String),
    Unset(&'static str),
}

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: HashMap<&'static str, Option<String>>,
}

impl EnvGuard {
    fn apply<const N: usize>(changes: [EnvChange; N]) -> Self {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        let lock = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock");

        let mut previous = HashMap::new();
        for change in changes {
            let key = match &change {
                EnvChange::Set(key, _) | EnvChange::Unset(key) => *key,
            };
            previous
                .entry(key)
                .or_insert_with(|| std::env::var(key).ok());

            match change {
                EnvChange::Set(key, value) => unsafe {
                    std::env::set_var(key, value);
                },
                EnvChange::Unset(key) => unsafe {
                    std::env::remove_var(key);
                },
            }
        }

        Self {
            _lock: lock,
            previous,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            match value {
                Some(value) => unsafe {
                    std::env::set_var(key, value);
                },
                None => unsafe {
                    std::env::remove_var(key);
                },
            }
        }
    }
}
