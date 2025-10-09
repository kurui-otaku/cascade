use crate::domain::{
    error::{DomainError, RepositoryError},
    models::{credential::HashedPassword, user::User},
    repositories::{credential_repository::CredentialRepository, user_repository::UserRepository},
    services::token_service::TokenGenerator,
};

#[derive(Debug)]
pub struct LoginResult {
    pub token: String,
    pub user: User,
}

pub struct LoginUsecase<C: CredentialRepository, U: UserRepository, T: TokenGenerator> {
    credential_repository: C,
    user_repository: U,
    token_generator: T,
}

impl<C: CredentialRepository, U: UserRepository, T: TokenGenerator> LoginUsecase<C, U, T> {
    pub fn new(credential_repository: C, user_repository: U, token_generator: T) -> Self {
        Self {
            credential_repository,
            user_repository,
            token_generator,
        }
    }

    pub async fn login(&self, user_id: String, password: String) -> Result<LoginResult, DomainError>
    where
        C: Send + Sync,
        U: Send + Sync,
        T: Send + Sync,
    {
        let password_hash = HashedPassword::from_password(&password)?;

        let credential = self
            .credential_repository
            .get_credential(user_id, password_hash)
            .await?;

        let user = self
            .user_repository
            .find_by_id(credential.id())
            .await?
            .ok_or(RepositoryError::NotFound)?;

        let token = self.token_generator.generate(&user)?;

        Ok(LoginResult { token, user })
    }
}
