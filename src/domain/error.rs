use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Weak password (minimum 8 characters required)")]
    WeakPassword,

    #[error("Empty display name")]
    EmptyDisplayName,

    #[error("Invalid activity ID")]
    InvalidActivityId,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(String),
}
