use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::domain::{
    error::RepositoryError,
    models::credential::{Credential, HashedPassword},
    repositories::credential_repository::CredentialRepository,
};
use entity::credentials;

#[derive(Clone)]
pub struct PostgresCredentialRepository {
    db: DatabaseConnection,
}

impl PostgresCredentialRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CredentialRepository for PostgresCredentialRepository {
    async fn get_credential(
        &self,
        user_id: String,
        password_hash: HashedPassword,
    ) -> Result<Credential, RepositoryError> {
        let credential = credentials::Entity::find()
            .filter(credentials::Column::ActivityId.eq(user_id.as_str()))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        let password_hash_reconstructed = HashedPassword::from_string(credential.password_hash)
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // パスワードハッシュの検証
        if !password_hash_reconstructed.verify(password_hash.as_str()) {
            return Err(RepositoryError::NotFound);
        }

        let credential = Credential::reconstruct(
            credential.user_id,
            user_id,
            password_hash_reconstructed,
            credential.created_at.naive_utc().and_utc(),
            credential.updated_at.naive_utc().and_utc(),
        );

        Ok(credential)
    }
}
