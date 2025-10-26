use async_trait::async_trait;

use crate::domain::{
    error::RepositoryError,
    models::{
        credential::HashedPassword,
        user::{ActivityId, User},
    },
};

/// Repository for user registration that handles both user and credential creation atomically
#[async_trait]
pub trait UserRegistrationRepository {
    /// Register a new user with credentials in a single transaction
    async fn register_user_with_credentials(
        &self,
        activity_id: &ActivityId,
        display_name: &str,
        password_hash: HashedPassword,
        email: String,
    ) -> Result<User, RepositoryError>;
}
