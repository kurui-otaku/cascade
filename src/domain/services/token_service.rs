use async_trait::async_trait;

use crate::domain::{error::DomainError, models::user::User};

pub type Token = String;

#[async_trait]
pub trait TokenGenerator: Send + Sync {
    fn generate(&self, user: &User) -> Result<Token, DomainError>;
}
