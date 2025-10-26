use crate::{
    domain::{
        error::DomainError,
        models::user::{ActivityId, User},
        repositories::{
            credential_repository::CredentialRepository, user_repository::UserRepository,
        },
        services::{password_service::PasswordHasher, token_service::TokenGenerator},
    },
    usecase::login_usecase::LoginResult,
};

pub struct RegisterUserUsecase<
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
    RegisterUserUsecase<C, U, P, T>
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

    pub async fn create_user(
        &self,
        user_id: String,
        display_name: String,
        password: String,
        email: String,
    ) -> Result<LoginResult, DomainError>
    where
        C: Send + Sync,
        U: Send + Sync,
        P: Send + Sync,
        T: Send + Sync,
    {
        // Generate ActivityId from username
        let instance_host = std::env::var("INSTANCE_HOST")
            .unwrap_or_else(|_| "example.com".to_string());
        let activity_id_str = format!("https://{}/users/{}", instance_host, user_id);
        let activity_id = ActivityId::new(activity_id_str)?;

        let password_hash = self.password_hasher.hash(&password)?;
        let id = self
            .user_repository
            .register_user(&activity_id, &display_name)
            .await?;
        let user = User::new(id, activity_id, display_name, None)?;
        self.credential_repository
            .create_credential(user.id(), user.activity_id().clone(), password_hash, email)
            .await?;
        let token = self.token_generator.generate(&user)?;
        Ok(LoginResult { token, user })
    }
}
