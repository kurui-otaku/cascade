use crate::domain::{error::DomainError, models::credential::HashedPassword};

/// Service for hashing and verifying passwords
pub trait PasswordHasher: Clone {
    /// Hash a plain text password
    fn hash(&self, plain_password: &str) -> Result<HashedPassword, DomainError>;

    /// Verify a plain text password against a hashed password
    fn verify(&self, plain_password: &str, hashed_password: &HashedPassword) -> Result<bool, DomainError>;
}
