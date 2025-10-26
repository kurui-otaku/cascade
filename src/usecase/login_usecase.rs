use crate::domain::{
    error::{DomainError, RepositoryError},
    models::user::User,
    repositories::{credential_repository::CredentialRepository, user_repository::UserRepository},
    services::{
        password_service::PasswordHasher,
        token_service::{Token, TokenGenerator},
    },
};

#[derive(Debug)]
pub struct LoginResult {
    pub token: Token,
    pub user: User,
}

pub struct LoginUsecase<
    C: CredentialRepository,
    U: UserRepository,
    P: PasswordHasher,
    T: TokenGenerator,
> {
    credential_repository: C,
    user_repository: U,
    password_hasher: P,
    token_generator: T,
}

impl<C: CredentialRepository, U: UserRepository, P: PasswordHasher, T: TokenGenerator>
    LoginUsecase<C, U, P, T>
{
    pub fn new(
        credential_repository: C,
        user_repository: U,
        password_hasher: P,
        token_generator: T,
    ) -> Self {
        Self {
            credential_repository,
            user_repository,
            password_hasher,
            token_generator,
        }
    }

    pub async fn login(&self, user_id: String, password: String) -> Result<LoginResult, DomainError>
    where
        C: Send + Sync,
        U: Send + Sync,
        P: Send + Sync,
        T: Send + Sync,
    {
        // Get credential from repository
        let credential = self.credential_repository.get_credential(user_id).await?;

        // Verify password using PasswordHasher
        let is_valid = self
            .password_hasher
            .verify(&password, credential.password_hash())?;
        credential.validate(is_valid)?;

        // Get user from repository
        let user = self
            .user_repository
            .find_by_id(credential.id())
            .await?
            .ok_or(RepositoryError::NotFound)?;

        // Generate token
        let token = self.token_generator.generate(&user)?;

        Ok(LoginResult { token, user })
    }
}
