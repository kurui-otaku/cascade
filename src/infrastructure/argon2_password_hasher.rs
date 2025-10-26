use argon2::{
    Argon2, PasswordHash as Argon2Hash,
    password_hash::{PasswordHasher as Argon2Hasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::domain::{
    error::DomainError,
    models::credential::HashedPassword,
    services::password_service::PasswordHasher,
};

#[derive(Clone)]
pub struct Argon2PasswordHasher;

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, plain_password: &str) -> Result<HashedPassword, DomainError> {
        // Validate password strength
        if plain_password.len() < 8 {
            return Err(DomainError::WeakPassword);
        }

        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(plain_password.as_bytes(), &salt)
            .map_err(|_| DomainError::InvalidCredentials)?
            .to_string();

        Ok(HashedPassword::new(hash))
    }

    fn verify(&self, plain_password: &str, hashed_password: &HashedPassword) -> Result<bool, DomainError> {
        let parsed_hash = Argon2Hash::new(hashed_password.as_str())
            .map_err(|_| DomainError::InvalidCredentials)?;

        Ok(Argon2::default()
            .verify_password(plain_password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
