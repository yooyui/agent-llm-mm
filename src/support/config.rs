use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const DATABASE_URL_ENV_VAR: &str = "AGENT_LLM_MM_DATABASE_URL";
pub const CONFIG_PATH_ENV_VAR: &str = "AGENT_LLM_MM_CONFIG";
pub const DEFAULT_CONFIG_FILE_NAME: &str = "agent-llm-mm.local.toml";
const DEFAULT_OPENAI_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransportKind {
    Stdio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelProviderKind {
    Mock,
    #[serde(rename = "openai-compatible")]
    OpenAiCompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiCompatibleConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelConfig {
    Mock,
    OpenAiCompatible(OpenAiCompatibleConfig),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub transport: TransportKind,
    pub database_url: String,
    pub model_provider: ModelProviderKind,
    pub model_config: ModelConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transport: TransportKind::Stdio,
            database_url: default_database_url(),
            model_provider: ModelProviderKind::Mock,
            model_config: ModelConfig::Mock,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        match std::env::var(CONFIG_PATH_ENV_VAR) {
            Ok(path) => Self::load_from_path(path),
            Err(_) => {
                let default_path = PathBuf::from(DEFAULT_CONFIG_FILE_NAME);
                if default_path.exists() {
                    Self::load_from_path(default_path)
                } else {
                    Ok(Self::default())
                }
            }
        }
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read config file {}: {error}", path.display()))?;
        let file_config: FileConfig = toml::from_str(&content)
            .map_err(|error| format!("failed to parse config file {}: {error}", path.display()))?;

        let mut config = Self::default();

        if let Some(transport) = file_config.transport {
            config.transport = transport;
        }
        if let Some(database_url) = file_config.database_url {
            config.database_url = database_url;
        }
        if let Some(model) = file_config.model {
            let provider = model.provider.unwrap_or(ModelProviderKind::Mock);
            config.model_provider = provider;
            config.model_config = match provider {
                ModelProviderKind::Mock => ModelConfig::Mock,
                ModelProviderKind::OpenAiCompatible => {
                    let openai = model.openai_compatible.unwrap_or_default();
                    ModelConfig::OpenAiCompatible(OpenAiCompatibleConfig {
                        base_url: openai.base_url.unwrap_or_default(),
                        api_key: openai.api_key.unwrap_or_default(),
                        model: openai.model.unwrap_or_default(),
                        timeout_ms: openai.timeout_ms.unwrap_or(DEFAULT_OPENAI_TIMEOUT_MS),
                    })
                }
            };
        }

        Ok(config)
    }

    pub fn validate_model_config(&self) -> Result<(), String> {
        match (&self.model_provider, &self.model_config) {
            (ModelProviderKind::Mock, ModelConfig::Mock) => Ok(()),
            (ModelProviderKind::Mock, _) => {
                Err("model provider is mock but model config is not mock".to_string())
            }
            (ModelProviderKind::OpenAiCompatible, ModelConfig::OpenAiCompatible(config)) => {
                if config.base_url.trim().is_empty() {
                    return Err("missing required openai-compatible field: base_url".to_string());
                }
                if config.api_key.trim().is_empty() {
                    return Err("missing required openai-compatible field: api_key".to_string());
                }
                if config.model.trim().is_empty() {
                    return Err("missing required openai-compatible field: model".to_string());
                }
                Ok(())
            }
            (ModelProviderKind::OpenAiCompatible, _) => Err(
                "model provider is openai-compatible but model config is not openai-compatible"
                    .to_string(),
            ),
        }
    }

    pub fn doctor_model(&self) -> Option<String> {
        match &self.model_config {
            ModelConfig::Mock => None,
            ModelConfig::OpenAiCompatible(config) => Some(config.model.clone()),
        }
    }

    pub fn doctor_base_url(&self) -> Option<String> {
        match &self.model_config {
            ModelConfig::Mock => None,
            ModelConfig::OpenAiCompatible(config) => Some(config.base_url.clone()),
        }
    }
}

fn default_database_url() -> String {
    std::env::var(DATABASE_URL_ENV_VAR)
        .unwrap_or_else(|_| sqlite_url(&std::env::temp_dir().join("agent-llm-mm.sqlite")))
}

fn sqlite_url(path: &Path) -> String {
    format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    transport: Option<TransportKind>,
    database_url: Option<String>,
    model: Option<FileModelConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct FileModelConfig {
    provider: Option<ModelProviderKind>,
    #[serde(default)]
    openai_compatible: Option<FileOpenAiCompatibleConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct FileOpenAiCompatibleConfig {
    base_url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    timeout_ms: Option<u64>,
}
