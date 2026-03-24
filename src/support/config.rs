use std::path::Path;

pub const DATABASE_URL_ENV_VAR: &str = "AGENT_LLM_MM_DATABASE_URL";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Stdio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub transport: TransportKind,
    pub database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transport: TransportKind::Stdio,
            database_url: default_database_url(),
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
