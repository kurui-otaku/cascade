use std::sync::Arc;

use crate::{
    domain::{
        repositories::{
            credential_repository::CredentialRepository, user_repository::UserRepository,
        },
        services::token_service::TokenGenerator,
    },
    usecase::login_usecase::LoginUsecase,
};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState<C: CredentialRepository, U: UserRepository, T: TokenGenerator> {
    pub login_service: Arc<LoginUsecase<C, U, T>>,
}

pub fn create_user_router<
    C: CredentialRepository + Send + Sync + 'static + Clone,
    U: UserRepository + Send + Sync + 'static + Clone,
    T: TokenGenerator + Send + Sync + 'static + Clone,
>(
    login_service: LoginUsecase<C, U, T>,
) -> Router {
    let state = AppState {
        login_service: Arc::new(login_service),
    };

    Router::new()
        .route("/login", post(login::<C, U, T>))
        .with_state(state)
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub user_id: String,
    pub password: String,
}

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
        // activity_idのURLからユーザー名を抽出
        // 例: "https://example.com/users/user123" -> "user123"
        let username = user
            .activity_id()
            .as_str()
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();

        // activity_idからホストを抽出してacct形式を生成
        // ローカルユーザーの場合: "username"
        // リモートユーザーの場合: "username@domain.com"
        let acct = if let Some(host) = extract_host(user.activity_id().as_str()) {
            // 自インスタンスのホストと比較（環境変数から取得）
            let self_host = std::env::var("INSTANCE_HOST")
                .unwrap_or_else(|_| "example.com".to_string())
                .to_string();

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

// URLからホスト部分を抽出するヘルパー関数
fn extract_host(url: &str) -> Option<String> {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .and_then(|s| s.split('/').next())
        .map(|s| s.to_string())
}

async fn login<
    C: CredentialRepository + Send + Sync,
    U: UserRepository + Send + Sync,
    T: TokenGenerator + Send + Sync,
>(
    State(state): State<AppState<C, U, T>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // LoginUsecaseを実行
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
