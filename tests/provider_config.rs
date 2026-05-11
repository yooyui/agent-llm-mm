use agent_llm_mm::{
    run_doctor,
    support::config::{
        AppConfig, DATABASE_URL_ENV_VAR, ModelConfig, ModelProviderKind, OpenAiCompatibleConfig,
        TransportKind,
    },
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
use tempfile::tempdir;

#[test]
fn default_config_uses_mock_provider_when_no_config_file_is_present() {
    let config = AppConfig::default();

    assert_eq!(config.model_provider, ModelProviderKind::Mock);
    assert_eq!(config.model_config, ModelConfig::Mock);
}

#[test]
fn load_from_path_reads_openai_compatible_provider_from_toml_file() {
    let temp_dir = tempdir().expect("temp dir");
    let config_path = temp_dir.path().join("agent-llm-mm.local.toml");
    fs::write(
        &config_path,
        r#"
transport = "stdio"
database_url = "sqlite:///D:/back/agent-llm-mm-test.sqlite"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "https://api.example.test/v1"
api_key = "example-test-key"
model = "gpt-4o-mini"
timeout_ms = 45000
"#,
    )
    .expect("write config");
    let _guard = EnvGuard::set([(DATABASE_URL_ENV_VAR, Some("sqlite:///tmp/from-env.sqlite"))]);

    let config = AppConfig::load_from_path(&config_path).expect("config");

    assert_eq!(config.transport, TransportKind::Stdio);
    assert_eq!(
        config.database_url,
        "sqlite:///D:/back/agent-llm-mm-test.sqlite"
    );
    assert_eq!(config.model_provider, ModelProviderKind::OpenAiCompatible);
    assert_eq!(
        config.model_config,
        ModelConfig::OpenAiCompatible(OpenAiCompatibleConfig {
            base_url: "https://api.example.test/v1".to_string(),
            api_key: "example-test-key".to_string(),
            model: "gpt-4o-mini".to_string(),
            timeout_ms: 45_000,
        })
    );
}

#[test]
fn load_prefers_config_path_from_environment() {
    let temp_dir = tempdir().expect("temp dir");
    let config_path = temp_dir.path().join("custom-provider.toml");
    fs::write(
        &config_path,
        r#"
[model]
provider = "mock"
"#,
    )
    .expect("write config");

    let _guard = EnvGuard::set([(
        "AGENT_LLM_MM_CONFIG",
        Some(config_path.to_string_lossy().as_ref()),
    )]);

    let config = AppConfig::load().expect("config");

    assert_eq!(config.model_provider, ModelProviderKind::Mock);
    assert_eq!(config.model_config, ModelConfig::Mock);
}

#[test]
fn load_prefers_database_url_env_over_default_config_file() {
    let temp_dir = tempdir().expect("temp dir");
    let config_path = temp_dir.path().join("agent-llm-mm.local.toml");
    fs::write(
        &config_path,
        r#"
transport = "stdio"
database_url = "sqlite:///tmp/from-config-file.sqlite"

[model]
provider = "mock"
"#,
    )
    .expect("write config");

    let _guard = ProcessContextGuard::apply(
        temp_dir.path(),
        [
            ("AGENT_LLM_MM_CONFIG", None),
            (DATABASE_URL_ENV_VAR, Some("sqlite:///tmp/from-env.sqlite")),
        ],
    );

    let config = AppConfig::load().expect("config");

    assert_eq!(config.database_url, "sqlite:///tmp/from-env.sqlite");
    assert_eq!(config.model_provider, ModelProviderKind::Mock);
}

#[test]
fn dev_example_config_parses_without_real_secrets() {
    let config = load_example_config("agent-llm-mm.dev.example.toml");

    assert_eq!(config.transport, TransportKind::Stdio);
    assert_eq!(config.model_provider, ModelProviderKind::Mock);
    assert_eq!(config.model_config, ModelConfig::Mock);
    assert!(!config.dashboard.enabled);
    assert!(!config.daemon.enabled);
    config.validate().expect("dev example config should validate");
}

#[test]
fn prod_local_example_config_parses_with_local_dashboard_and_disabled_daemon() {
    let config = load_example_config("agent-llm-mm.prod-local.example.toml");

    assert_eq!(config.transport, TransportKind::Stdio);
    assert_eq!(config.model_provider, ModelProviderKind::OpenAiCompatible);
    let ModelConfig::OpenAiCompatible(provider_config) = &config.model_config else {
        panic!("prod-local example should use openai-compatible provider settings");
    };
    assert_eq!(provider_config.api_key, "REPLACE_WITH_LOCAL_SECRET");
    assert!(
        !provider_config.api_key.starts_with("sk-"),
        "prod-local example must not contain a live-looking API key"
    );
    assert_eq!(config.dashboard.host, "127.0.0.1");
    assert!(!config.daemon.enabled);
    config
        .validate()
        .expect("prod-local example config structure should validate");
}

#[test]
fn generic_example_config_parses_and_keeps_daemon_disabled() {
    let config = load_example_config("agent-llm-mm.example.toml");

    assert_eq!(config.transport, TransportKind::Stdio);
    assert!(!config.daemon.enabled);
    config
        .validate()
        .expect("generic example config should validate");
}

#[tokio::test]
async fn doctor_fails_when_openai_provider_config_is_missing_api_key() {
    let temp_dir = tempdir().expect("temp dir");
    let database_url = sqlite_url(temp_dir.path().join("doctor.sqlite"));
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url,
        model_provider: ModelProviderKind::OpenAiCompatible,
        model_config: ModelConfig::OpenAiCompatible(OpenAiCompatibleConfig {
            base_url: "https://api.example.test/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            timeout_ms: 30_000,
        }),
        dashboard: Default::default(),
        ..Default::default()
    };

    let error = run_doctor(config).await.expect_err("doctor should fail");

    assert!(error.to_string().contains("api_key"));
}

#[tokio::test]
async fn doctor_report_does_not_contain_api_key_in_serialized_output() {
    let temp_dir = tempdir().expect("temp dir");
    let database_url = sqlite_url(temp_dir.path().join("doctor-redact.sqlite"));
    let secret = "sk-super-secret-key-12345";
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url,
        model_provider: ModelProviderKind::OpenAiCompatible,
        model_config: ModelConfig::OpenAiCompatible(OpenAiCompatibleConfig {
            base_url: "https://api.example.test/v1".to_string(),
            api_key: secret.to_string(),
            model: "gpt-4o-mini".to_string(),
            timeout_ms: 30_000,
        }),
        dashboard: Default::default(),
        ..Default::default()
    };

    let report = run_doctor(config).await.expect("doctor");
    let json = serde_json::to_string(&report).expect("serialize");

    assert!(
        !json.contains(secret),
        "doctor output must not contain the api_key secret"
    );
}

fn sqlite_url(path: PathBuf) -> String {
    format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
}

fn load_example_config(file_name: &str) -> AppConfig {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(file_name);
    AppConfig::load_from_path(path).expect("example config should parse")
}

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: HashMap<&'static str, Option<String>>,
}

impl EnvGuard {
    fn set<const N: usize>(pairs: [(&'static str, Option<&str>); N]) -> Self {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        let lock = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock");

        let mut previous = HashMap::new();
        for (key, value) in pairs {
            previous.insert(key, std::env::var(key).ok());
            match value {
                Some(value) => unsafe {
                    std::env::set_var(key, value);
                },
                None => unsafe {
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

struct ProcessContextGuard {
    _env: EnvGuard,
    previous_dir: PathBuf,
}

impl ProcessContextGuard {
    fn apply<const N: usize>(dir: &Path, pairs: [(&'static str, Option<&str>); N]) -> Self {
        let env_guard = EnvGuard::set(pairs);
        let previous_dir = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(dir).expect("set current dir");

        Self {
            _env: env_guard,
            previous_dir,
        }
    }
}

impl Drop for ProcessContextGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.previous_dir).expect("restore current dir");
    }
}
