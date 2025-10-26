use crate::domain::{
    error::RepositoryError,
    models::user::{ActivityId, User},
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError>;
    async fn register_user(
        &self,
        activity_id: &ActivityId,
        display_name: &str,
    ) -> Result<Uuid, RepositoryError>;
}
