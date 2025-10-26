use async_trait::async_trait;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, TransactionTrait};
use uuid::Uuid;

use crate::domain::{
    error::RepositoryError,
    models::{
        credential::HashedPassword,
        user::{ActivityId, User},
    },
    repositories::user_registration_repository::UserRegistrationRepository,
};
use entity::{credentials, users};

#[derive(Clone)]
pub struct PostgresUserRegistrationRepository {
    db: DatabaseConnection,
}

impl PostgresUserRegistrationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRegistrationRepository for PostgresUserRegistrationRepository {
    async fn register_user_with_credentials(
        &self,
        activity_id: &ActivityId,
        display_name: &str,
        password_hash: HashedPassword,
        email: String,
    ) -> Result<User, RepositoryError> {
        // Begin transaction
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let user_id = Uuid::new_v4();

        // Insert user
        let user_model = users::ActiveModel {
            id: Set(user_id),
            activity_id: Set(activity_id.as_str().to_string()),
            name: Set(display_name.to_string()),
            summary: Set(String::new()),
            icon: Set(None),
        };

        users::Entity::insert(user_model)
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Insert credential
        let now = chrono::Utc::now().fixed_offset();
        let credential_model = credentials::ActiveModel {
            user_id: Set(user_id),
            activity_id: Set(activity_id.as_str().to_string()),
            password_hash: Set(password_hash.as_str().to_string()),
            email: Set(email),
            created_at: Set(now),
            updated_at: Set(now),
        };

        credentials::Entity::insert(credential_model)
            .exec(&txn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Commit transaction
        txn.commit()
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Construct domain model
        let user = User::new(user_id, activity_id.clone(), display_name.to_string(), None)
            .expect("Failed to create User from validated data");

        Ok(user)
    }
}
