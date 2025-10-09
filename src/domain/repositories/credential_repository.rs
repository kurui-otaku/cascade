use async_trait::async_trait;

use crate::domain::{
    error::RepositoryError,
    models::credential::{Credential, HashedPassword},
};

#[async_trait]
pub trait CredentialRepository {
    async fn get_credential(
        &self,
        user_id: String,
        password_hash: HashedPassword,
    ) -> Result<Credential, RepositoryError>;
}
