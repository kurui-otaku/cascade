mod domain;
mod infrastructure;
mod presentation;
mod usecase;

use axum::{Router, routing::get};
use sea_orm::{ConnectOptions, Database};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::{
    infrastructure::{
        credential_repository::PostgresCredentialRepository,
        jwt_token_generator::JwtTokenGenerator, user_repository::PostgresUserRepository,
    },
    presentation::handlers::user_handler::create_user_router,
    usecase::login_usecase::LoginUsecase,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_path("../.env")?;
    let mut opt = ConnectOptions::new(dotenvy::var("DATABASE_URL")?);
    opt.max_connections(10)
        .min_connections(1)
        .sqlx_logging(true);

    let db = Database::connect(opt)
        .await
        .expect("Connection to DB failed");
    let user_repository = PostgresUserRepository::new(db.clone());
    let credential_repository = PostgresCredentialRepository::new(db.clone());
    let token_generator = JwtTokenGenerator::new("testtoken".to_string());
    let login_service = LoginUsecase::new(credential_repository, user_repository, token_generator);

    let app = Router::new()
        .route("/", get(|| async { "Hello, Axum!!!" }))
        .nest("/api", create_user_router(login_service));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use rstest::*;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        domain::{
            error::{DomainError, RepositoryError},
            models::{
                credential::{Credential, HashedPassword},
                user::{ActivityId, User},
            },
            repositories::{
                credential_repository::CredentialRepository, user_repository::UserRepository,
            },
            services::token_service::TokenGenerator,
        },
        presentation::handlers::user_handler::{LoginRequest, LoginResponse, create_user_router},
        usecase::login_usecase::LoginUsecase,
    };

    const TEST_ID: &str = "00000000-0000-0000-0000-000000000001";

    // mock repository interface
    #[derive(Clone)]
    struct MockCredentialRepository;

    #[async_trait]
    impl CredentialRepository for MockCredentialRepository {
        async fn get_credential(
            &self,
            user_id: String,
            password_hash: HashedPassword,
        ) -> Result<Credential, RepositoryError> {
            if user_id.as_str() == "testuser" {
                let id = Uuid::parse_str(TEST_ID).unwrap();
                Ok(Credential::new(id, user_id, password_hash))
            } else {
                Err(RepositoryError::NotFound)
            }
        }
    }

    #[derive(Clone)]
    struct MockUserRepository;

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
            // "testuser"用のユーザー情報を返す
            if username == "testuser" {
                let id = Uuid::parse_str(TEST_ID).unwrap();
                let activity_id =
                    ActivityId::new("https://example.com/users/testuser".to_string()).unwrap();
                let user = User::new(id, activity_id, "testuser".to_string(), None).unwrap();
                Ok(Some(user))
            } else {
                Ok(None)
            }
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
            if id.to_string() == TEST_ID {
                let user_id = Uuid::parse_str(TEST_ID).unwrap();
                let activity_id =
                    ActivityId::new("https://example.com/users/testuser".to_string()).unwrap();
                let user = User::new(user_id, activity_id, "testuser".to_string(), None).unwrap();
                Ok(Some(user))
            } else {
                Ok(None)
            }
        }
    }

    #[derive(Clone)]
    struct MockTokenGenerator;

    impl TokenGenerator for MockTokenGenerator {
        fn generate(&self, _user: &User) -> Result<String, DomainError> {
            Ok("mock_token".to_string())
        }
    }

    #[fixture]
    async fn test_app() -> Router {
        // set up mock repository
        let mock_credential_repo = MockCredentialRepository;
        let mock_user_repo = MockUserRepository;
        let mock_token_generator = MockTokenGenerator;

        // create service of LoginUsecase
        let login_usecase =
            LoginUsecase::new(mock_credential_repo, mock_user_repo, mock_token_generator);

        // setup router: sync settings of main.app
        Router::new().nest("/api", create_user_router(login_usecase))
    }

    #[rstest]
    #[tokio::test]
    async fn test_login(#[future] test_app: Router) {
        let app = test_app.await;

        let user_id = "testuser".to_string();
        let password = "test_password".to_string();
        let login_request = LoginRequest {
            user_id: user_id.clone(),
            password: password.clone(),
        };
        let body = serde_json::to_string(&login_request).unwrap();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let login_response: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(TEST_ID, login_response.user.id);
        assert_eq!(user_id, login_response.user.acct);
    }
}
