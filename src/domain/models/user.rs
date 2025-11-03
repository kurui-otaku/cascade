use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;

pub type IconUrl = String;
pub type DisplayName = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(Uuid);
impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityId(String);
impl ActivityId {
    pub fn new(value: String) -> Result<Self, DomainError> {
        if !value.starts_with("https://") {
            return Err(DomainError::InvalidActivityId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    activity_id: ActivityId,
    display_name: DisplayName,
    icon_url: Option<IconUrl>,
}

impl User {
    pub fn new(
        id: Uuid,
        activity_id: ActivityId,
        display_name: DisplayName,
        icon_url: Option<IconUrl>,
    ) -> Result<Self, DomainError> {
        if display_name.is_empty() {
            return Err(DomainError::EmptyDisplayName);
        }

        let id = UserId(id);
        Ok(Self {
            id,
            activity_id,
            display_name,
            icon_url,
        })
    }

    // getterのみ提供
    pub fn id(&self) -> &UserId {
        &self.id
    }
    pub fn activity_id(&self) -> &ActivityId {
        &self.activity_id
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    pub fn icon_url(&self) -> Option<&str> {
        self.icon_url.as_deref()
    }
}
