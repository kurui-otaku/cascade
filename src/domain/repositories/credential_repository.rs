use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    error::RepositoryError,
    models::{
        credential::{Credential, HashedPassword},
        user::ActivityId,
    },
};

#[async_trait]
pub trait CredentialRepository {
    async fn get_credential(&self, user_id: String) -> Result<Credential, RepositoryError>;
    async fn create_credential(
        &self,
        id: Uuid,
        user_id: ActivityId,
        password_hash: HashedPassword,
        email: String,
    ) -> Result<(), RepositoryError>;
}
