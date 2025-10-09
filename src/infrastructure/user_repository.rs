use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::{
    error::RepositoryError,
    models::user::{ActivityId, User},
    repositories::user_repository::UserRepository,
};
use entity::users;

#[derive(Clone)]
pub struct PostgresUserRepository {
    db: DatabaseConnection,
}

impl PostgresUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let user = users::Entity::find()
            .filter(users::Column::Name.eq(username))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match user {
            Some(model) => {
                let activity_id = ActivityId::new(model.activity_id)
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                let icon_url = model.icon.as_ref().and_then(|icon| {
                    icon.as_object()
                        .and_then(|obj| obj.get("url"))
                        .and_then(|url| url.as_str())
                        .map(|s| s.to_string())
                });

                let user = User::new(model.id, activity_id, model.name, icon_url)
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match user {
            Some(model) => {
                let activity_id = ActivityId::new(model.activity_id)
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                let icon_url = model.icon.as_ref().and_then(|icon| {
                    icon.as_object()
                        .and_then(|obj| obj.get("url"))
                        .and_then(|url| url.as_str())
                        .map(|s| s.to_string())
                });

                let user = User::new(model.id, activity_id, model.name, icon_url)
                    .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}
