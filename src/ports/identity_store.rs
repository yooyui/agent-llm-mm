use async_trait::async_trait;

use crate::{domain::identity_core::IdentityCore, error::AppError};

#[async_trait]
pub trait IdentityStore {
    async fn load_identity(&self) -> Result<IdentityCore, AppError>;
    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError>;
}
