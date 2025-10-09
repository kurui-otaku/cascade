use argon2::{
    Argon2, PasswordHash as Argon2Hash,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn from_password(plain_password: &str) -> Result<Self, DomainError> {
        if plain_password.len() < 8 {
            return Err(DomainError::WeakPassword);
        }

        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(plain_password.as_bytes(), &salt)
            .map_err(|_| DomainError::InvalidCredentials)?
            .to_string();

        Ok(Self(hash))
    }

    pub fn from_string(hash: String) -> Result<Self, DomainError> {
        Argon2Hash::new(&hash).map_err(|_| DomainError::InvalidCredentials)?;
        Ok(Self(hash))
    }

    pub fn verify(&self, plain_password: &str) -> bool {
        let parsed_hash = match Argon2Hash::new(&self.0) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(plain_password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Credential {
    id: Uuid,
    user_id: String,
    password_hash: HashedPassword,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Credential {
    pub fn new(id: Uuid, user_id: String, password_hash: HashedPassword) -> Self {
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
        user_id: String,
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

    pub fn verify_password(&self, plain_password: &str) -> Result<(), DomainError> {
        if self.password_hash.verify(plain_password) {
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

    pub fn user_id(&self) -> &String {
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
