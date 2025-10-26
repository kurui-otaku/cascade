use crate::{
    domain::{
        error::DomainError,
        models::user::ActivityId,
        repositories::user_registration_repository::UserRegistrationRepository,
        services::{password_service::PasswordHasher, token_service::TokenGenerator},
    },
    usecase::login_usecase::LoginResult,
};

pub struct RegisterUserUsecase<R: UserRegistrationRepository, P: PasswordHasher, T: TokenGenerator> {
    registration_repository: R,
    password_hasher: P,
    token_generator: T,
}

impl<R: UserRegistrationRepository, P: PasswordHasher, T: TokenGenerator>
    RegisterUserUsecase<R, P, T>
{
    pub fn new(
        registration_repository: R,
        password_hasher: P,
        token_generator: T,
    ) -> Self {
        Self {
            registration_repository,
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
        R: Send + Sync,
        P: Send + Sync,
        T: Send + Sync,
    {
        // Generate ActivityId from username
        let instance_host = std::env::var("INSTANCE_HOST")
            .unwrap_or_else(|_| "example.com".to_string());
        let activity_id_str = format!("https://{}/users/{}", instance_host, user_id);
        let activity_id = ActivityId::new(activity_id_str)?;

        // Hash password
        let password_hash = self.password_hasher.hash(&password)?;

        // Register user with credentials atomically
        let user = self
            .registration_repository
            .register_user_with_credentials(&activity_id, &display_name, password_hash, email)
            .await?;

        // Generate token
        let token = self.token_generator.generate(&user)?;

        Ok(LoginResult { token, user })
    }
}
