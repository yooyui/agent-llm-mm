use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
}

impl From<crate::domain::DomainError> for AppError {
    fn from(value: crate::domain::DomainError) -> Self {
        Self::Message(format!("{value:?}"))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Message(value.to_string())
    }
}
