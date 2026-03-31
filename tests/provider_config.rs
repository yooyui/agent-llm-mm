use agent_llm_mm::{
    run_doctor,
    support::config::{
        AppConfig, ModelConfig, ModelProviderKind, OpenAiCompatibleConfig, TransportKind,
    },
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
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
        config_path.to_string_lossy().as_ref(),
    )]);

    let config = AppConfig::load().expect("config");

    assert_eq!(config.model_provider, ModelProviderKind::Mock);
    assert_eq!(config.model_config, ModelConfig::Mock);
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
    };

    let error = run_doctor(config).await.expect_err("doctor should fail");

    assert!(error.to_string().contains("api_key"));
}

fn sqlite_url(path: PathBuf) -> String {
    format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
}

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: HashMap<&'static str, Option<String>>,
}

impl EnvGuard {
    fn set<const N: usize>(pairs: [(&'static str, &str); N]) -> Self {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        let lock = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock");

        let mut previous = HashMap::new();
        for (key, value) in pairs {
            previous.insert(key, std::env::var(key).ok());
            unsafe {
                std::env::set_var(key, value);
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
