use std::sync::Arc;

use crate::{
    domain::{
        repositories::{
            credential_repository::CredentialRepository,
            user_registration_repository::UserRegistrationRepository,
            user_repository::UserRepository,
        },
        services::{password_service::PasswordHasher, token_service::TokenGenerator},
    },
    usecase::{login_usecase::LoginUsecase, register_user_usecase::RegisterUserUsecase},
};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};

// Request

/// json for login request
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub user_id: String,
    pub password: String,
}

/// json for register request
#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub user_id: String,
    pub password: String,
    pub mail_address: String,
    pub display_name: String,
}

// Response

/// json for login response
#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub acct: String,
    pub display_name: String,
}

impl From<crate::domain::models::user::User> for UserInfo {
    fn from(user: crate::domain::models::user::User) -> Self {
        let username = user
            .activity_id()
            .as_str()
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();

        // generate acct format by host extracted from url
        // In the case of local user: "username"
        // In the case of remote user: "username@domain.com"
        let acct = if let Some(host) = extract_host(user.activity_id().as_str()) {
            // get instance host
            let self_host = std::env::var("INSTANCE_HOST")
                .unwrap_or_else(|_| "example.com".to_string())
                .to_string();
            // compare host by the host of instance
            if host == self_host {
                username.clone()
            } else {
                format!("{}@{}", username, host)
            }
        } else {
            username
        };

        Self {
            id: user.id().to_string(),
            acct,
            display_name: user.display_name().to_string(),
        }
    }
}

/// helper function that extract host from url
fn extract_host(url: &str) -> Option<String> {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .and_then(|s| s.split('/').next())
        .map(|s| s.to_string())
}

/* Router Function and Handler Function */

// User Router

/// function return Router object
/// Suppose to be nested by main router

pub fn create_user_router<
    C: CredentialRepository + Send + Sync + 'static + Clone,
    U: UserRepository + Send + Sync + 'static + Clone,
    R: UserRegistrationRepository + Send + Sync + 'static + Clone,
    P: PasswordHasher + Send + Sync + 'static + Clone,
    T: TokenGenerator + Send + Sync + 'static + Clone,
>(
    login_service: LoginUsecase<C, U, P, T>,
    register_service: RegisterUserUsecase<R, P, T>,
) -> Router {
    let state = AppState {
        login_service: Arc::new(login_service),
        register_service: Arc::new(register_service),
    };

    Router::new()
        .route("/login", post(login::<C, U, P, T>))
        .route("/register", post(register::<R, P, T>))
        .with_state(state)
}

#[derive(Clone)]
pub struct AppState<
    C: CredentialRepository,
    U: UserRepository,
    R: UserRegistrationRepository,
    P: PasswordHasher,
    T: TokenGenerator,
> {
    pub login_service: Arc<LoginUsecase<C, U, P, T>>,
    pub register_service: Arc<RegisterUserUsecase<R, P, T>>,
}

// handler function

/// handler function for login
async fn login<
    C: CredentialRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    P: PasswordHasher + Send + Sync,
    T: TokenGenerator + Send + Sync,
>(
    State(state): State<AppState<C, U, impl UserRegistrationRepository, P, T>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state
        .login_service
        .login(payload.user_id, payload.password)
        .await
    {
        Ok(result) => {
            let response = LoginResponse {
                token: result.token,
                user: result.user.into(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(_) => (StatusCode::UNAUTHORIZED, Json("Authentication failed")).into_response(),
    }
}

/// handler function for register
async fn register<
    R: UserRegistrationRepository + Send + Sync,
    P: PasswordHasher + Send + Sync,
    T: TokenGenerator + Send + Sync,
>(
    State(state): State<AppState<impl CredentialRepository, impl UserRepository, R, P, T>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match state
        .register_service
        .create_user(
            payload.user_id,
            payload.display_name,
            payload.password,
            payload.mail_address,
        )
        .await
    {
        Ok(result) => {
            let response = LoginResponse {
                token: result.token,
                user: result.user.into(),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json("Registration failed")).into_response(),
    }
}
