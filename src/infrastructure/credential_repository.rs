use async_trait::async_trait;
use chrono::Utc;
use entity::credentials;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::{
    error::RepositoryError,
    models::{
        credential::{Credential, HashedPassword},
        user::ActivityId,
    },
    repositories::credential_repository::CredentialRepository,
};

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
    async fn get_credential(&self, user_id: ActivityId) -> Result<Credential, RepositoryError> {
        let credential = credentials::Entity::find()
            .filter(credentials::Column::ActivityId.eq(user_id.as_str()))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        let password_hash = HashedPassword::new(credential.password_hash);

        let credential = Credential::reconstruct(
            credential.user_id,
            user_id,
            password_hash,
            credential.created_at.naive_utc().and_utc(),
            credential.updated_at.naive_utc().and_utc(),
        );

        Ok(credential)
    }
    async fn create_credential(
        &self,
        id: Uuid,
        activity_id: ActivityId,
        password_hash: HashedPassword,
        email: String,
    ) -> Result<(), RepositoryError> {
        let now = Utc::now();
        let credential = credentials::ActiveModel {
            user_id: Set(id),
            activity_id: Set(activity_id.as_str().to_string()),
            password_hash: Set(password_hash.as_str().to_string()),
            email: Set(email),
            created_at: Set(now.fixed_offset()),
            updated_at: Set(now.fixed_offset()),
        };
        credentials::Entity::insert(credential)
            .exec(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
