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
        argon2_password_hasher::Argon2PasswordHasher,
        credential_repository::PostgresCredentialRepository,
        jwt_token_generator::JwtTokenGenerator, user_repository::PostgresUserRepository,
    },
    presentation::handlers::user_handler::create_user_router,
    usecase::{
        login_usecase::LoginUsecase,
        register_user_usecase::{self, RegisterUserUsecase},
    },
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
    let password_hasher = Argon2PasswordHasher::new();
    let token_generator = JwtTokenGenerator::new("testtoken".to_string());
    let login_service = LoginUsecase::new(
        credential_repository.clone(),
        user_repository.clone(),
        password_hasher.clone(),
        token_generator.clone(),
    );
    let register_user_usecase = RegisterUserUsecase::new(
        credential_repository.clone(),
        user_repository.clone(),
        password_hasher.clone(),
        token_generator.clone(),
    );

    let app = Router::new()
        .route("/", get(|| async { "Hello, Axum!!!" }))
        .nest(
            "/api",
            create_user_router(login_service, register_user_usecase),
        );

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
        response::Response,
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
            services::{password_service::PasswordHasher, token_service::TokenGenerator},
        },
        presentation::handlers::user_handler::{
            LoginRequest, LoginResponse, RegisterRequest, create_user_router,
        },
        usecase::{login_usecase::LoginUsecase, register_user_usecase::RegisterUserUsecase},
    };

    const TEST_ID: &str = "00000000-0000-0000-0000-000000000001";

    // mock repository interface
    #[derive(Clone)]
    struct MockCredentialRepository;

    #[async_trait]
    impl CredentialRepository for MockCredentialRepository {
        async fn get_credential(&self, user_id: String) -> Result<Credential, RepositoryError> {
            if user_id.as_str() == "testuser" {
                let id = Uuid::parse_str(TEST_ID).unwrap();
                let password_hash = HashedPassword::new("mock_hash".to_string());
                Ok(Credential::new(id, user_id, password_hash))
            } else {
                Err(RepositoryError::NotFound)
            }
        }

        async fn create_credential(
            &self,
            _id: Uuid,
            _user_id: ActivityId,
            _password_hash: HashedPassword,
            email: String,
        ) -> Result<(), RepositoryError> {
            if email.contains("duplicated") {
                Err(RepositoryError::DatabaseError("Email already exists".to_string()))
            } else {
                Ok(())
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

        async fn register_user(
            &self,
            activity_id: &ActivityId,
            _display_name: &str,
        ) -> Result<Uuid, RepositoryError> {
            if activity_id.as_str().contains("duplicated_user") {
                Err(RepositoryError::DatabaseError("User already exists".to_string()))
            } else {
                Ok(Uuid::parse_str(TEST_ID).unwrap())
            }
        }
    }

    #[derive(Clone)]
    struct MockPasswordHasher;

    impl PasswordHasher for MockPasswordHasher {
        fn hash(&self, _plain_password: &str) -> Result<HashedPassword, DomainError> {
            Ok(HashedPassword::new("mock_hash".to_string()))
        }

        fn verify(
            &self,
            _plain_password: &str,
            _hashed_password: &HashedPassword,
        ) -> Result<bool, DomainError> {
            // Always return true for test
            Ok(true)
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
        let mock_password_hasher = MockPasswordHasher;
        let mock_token_generator = MockTokenGenerator;

        // create service of LoginUsecase
        let login_usecase = LoginUsecase::new(
            mock_credential_repo.clone(),
            mock_user_repo.clone(),
            mock_password_hasher.clone(),
            mock_token_generator.clone(),
        );

        // create service of RegisterUserUsecase
        let register_user_usecase = RegisterUserUsecase::new(
            mock_credential_repo.clone(),
            mock_user_repo.clone(),
            mock_password_hasher.clone(),
            mock_token_generator.clone(),
        );

        // setup router: sync settings of main.app
        Router::new().nest(
            "/api",
            create_user_router(login_usecase, register_user_usecase),
        )
    }

    // Login usecase

    /// # Description
    ///
    /// This function is general login handler
    /// Call this function from test case for login

    async fn login(app: Router, body: String) -> Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_positive(#[future] test_app: Router) {
        let app = test_app.await;

        // create request body
        let user_id = "testuser".to_string();
        let password = "test_password".to_string();
        let login_request = LoginRequest {
            user_id: user_id.clone(),
            password: password.clone(),
        };
        let body = serde_json::to_string(&login_request).unwrap();

        // send request
        let response = login(app, body).await;

        // validation
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let login_response: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(TEST_ID, login_response.user.id);
        assert_eq!(user_id, login_response.user.acct);
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_invalid_user_negative(#[future] test_app: Router) {
        let app = test_app.await;

        // create request body
        let user_id = "invalid_user".to_string();
        let password = "test_password".to_string();
        let login_request = LoginRequest {
            user_id: user_id.clone(),
            password: password.clone(),
        };
        let body = serde_json::to_string(&login_request).unwrap();

        // send request
        let response = login(app, body).await;

        // validation
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_invalid_password_negative(#[future] test_app: Router) {
        let app = test_app.await;

        // create request body
        let user_id = "test_password".to_string();
        let password = "test_password".to_string();
        let login_request = LoginRequest {
            user_id: user_id.clone(),
            password: password.clone(),
        };
        let body = serde_json::to_string(&login_request).unwrap();

        // send request
        let response = login(app, body).await;

        // validation
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Register usecase

    /// # Description
    ///
    /// This function is general register handler
    /// Call this function from test case for register
    async fn register(app: Router, body: String) -> Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_register_positive(#[future] test_app: Router) {
        let app = test_app.await;

        // create request body
        let new_user_id = "new_user";
        let new_password = "new_password";
        let new_mail_adress = "test@example.com";
        let new_display_name = "テスト";
        let register_request = RegisterRequest {
            user_id: new_user_id.to_string(),
            password: new_password.to_string(),
            mail_address: new_mail_adress.to_string(),
            display_name: new_display_name.to_string(),
        };
        let body = serde_json::to_string(&register_request).unwrap();

        // send request
        let response = register(app, body).await;

        // validation
        let status = response.status();
        if status != StatusCode::CREATED {
            let body = response.into_body();
            let bytes = body.collect().await.unwrap().to_bytes();
            let error_msg = String::from_utf8(bytes.to_vec()).unwrap();
            panic!("Expected CREATED but got {:?}. Error: {}", status, error_msg);
        }
        assert_eq!(status, StatusCode::CREATED);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let login_response: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(TEST_ID, login_response.user.id);
        assert_eq!(new_user_id, login_response.user.acct);
    }

    #[rstest]
    #[tokio::test]
    async fn test_register_duplicated_user_negative(#[future] test_app: Router) {
        let app = test_app.await;

        // create request body
        let new_user_id = "duplicated_user";
        let new_password = "new_password";
        let new_mail_adress = "test@example.com";
        let new_display_name = "テスト";
        let register_request = RegisterRequest {
            user_id: new_user_id.to_string(),
            password: new_password.to_string(),
            mail_address: new_mail_adress.to_string(),
            display_name: new_display_name.to_string(),
        };
        let body = serde_json::to_string(&register_request).unwrap();

        // send request
        let response = register(app, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[rstest]
    #[tokio::test]
    async fn test_register_duplicated_email_negative(#[future] test_app: Router) {
        let app = test_app.await;

        // create request body
        let new_user_id = "new_user";
        let new_password = "new_password";
        let new_mail_adress = "duplicated@example.com";
        let new_display_name = "テスト";
        let register_request = RegisterRequest {
            user_id: new_user_id.to_string(),
            password: new_password.to_string(),
            mail_address: new_mail_adress.to_string(),
            display_name: new_display_name.to_string(),
        };
        let body = serde_json::to_string(&register_request).unwrap();

        // send request
        let response = register(app, body).await;

        // validation
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
