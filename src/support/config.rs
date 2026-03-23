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
            database_url: "sqlite::memory:".to_string(),
        }
    }
}
