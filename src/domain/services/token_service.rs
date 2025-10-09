use async_trait::async_trait;

use crate::domain::{error::DomainError, models::user::User};

#[async_trait]
pub trait TokenGenerator: Send + Sync {
    fn generate(&self, user: &User) -> Result<String, DomainError>;
}
