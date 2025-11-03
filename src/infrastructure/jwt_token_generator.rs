use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::DomainError,
    models::user::User,
    services::token_service::{Token, TokenGenerator},
};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,         // Subject (user ID)
    activity_id: String, // Activity ID
    exp: i64,            // Expiration time
    iat: i64,            // Issued at
}

#[derive(Clone)]
pub struct JwtTokenGenerator {
    secret: String,
    expiration_hours: i64,
}

impl JwtTokenGenerator {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            expiration_hours: 24, // 24h
        }
    }

    pub fn with_expiration(secret: String, expiration_hours: i64) -> Self {
        Self {
            secret,
            expiration_hours,
        }
    }
}

impl TokenGenerator for JwtTokenGenerator {
    fn generate(&self, user: &User) -> Result<Token, DomainError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);

        let claims = Claims {
            sub: user.id().as_uuid().to_string(),
            activity_id: user.activity_id().as_str().to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| {
            DomainError::Repository(crate::domain::error::RepositoryError::DatabaseError(
                format!("Failed to generate token: {}", e),
            ))
        })
    }
}
