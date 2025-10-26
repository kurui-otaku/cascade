use chrono::{DateTime, Utc};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

use crate::domain::{error::DomainError, models::user::ActivityId};

/// Value object representing a hashed password
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedPassword(String);

impl HashedPassword {
    /// Create a new HashedPassword from an already hashed string
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    /// Get the hash as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Credential {
    id: Uuid,
    user_id: ActivityId,
    password_hash: HashedPassword,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Credential {
    pub fn new(id: Uuid, user_id: ActivityId, password_hash: HashedPassword) -> Self {
        let now = Utc::now();
        Self {
            id,
            user_id,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn reconstruct(
        id: Uuid,
        user_id: ActivityId,
        password_hash: HashedPassword,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            password_hash,
            created_at,
            updated_at,
        }
    }

    pub fn validate(&self, is_valid: bool) -> Result<(), DomainError> {
        if is_valid {
            Ok(())
        } else {
            Err(DomainError::AuthenticationFailed)
        }
    }

    pub fn change_password(&mut self, new_password_hash: HashedPassword) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> &ActivityId {
        &self.user_id
    }

    pub fn password_hash(&self) -> &HashedPassword {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
