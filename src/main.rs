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
        jwt_token_generator::JwtTokenGenerator,
        user_registration_repository::PostgresUserRegistrationRepository,
        user_repository::PostgresUserRepository,
    },
    presentation::handlers::user_handler::create_user_router,
    usecase::{login_usecase::LoginUsecase, register_user_usecase::RegisterUserUsecase},
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
    let registration_repository = PostgresUserRegistrationRepository::new(db.clone());
    let password_hasher = Argon2PasswordHasher::new();
    let token_generator = JwtTokenGenerator::new("testtoken".to_string());
    let login_service = LoginUsecase::new(
        credential_repository.clone(),
        user_repository.clone(),
        password_hasher.clone(),
        token_generator.clone(),
    );
    let register_user_usecase = RegisterUserUsecase::new(
        registration_repository,
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
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        response::Response,
    };
    use http_body_util::BodyExt;
    use sea_orm::{ActiveModelTrait, ConnectOptions, Database, Set};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        domain::services::password_service::PasswordHasher,
        infrastructure::{
            argon2_password_hasher::Argon2PasswordHasher,
            credential_repository::PostgresCredentialRepository,
            jwt_token_generator::JwtTokenGenerator,
            user_registration_repository::PostgresUserRegistrationRepository,
            user_repository::PostgresUserRepository,
        },
        presentation::handlers::user_handler::{
            LoginRequest, LoginResponse, RegisterRequest, create_user_router,
        },
        usecase::{login_usecase::LoginUsecase, register_user_usecase::RegisterUserUsecase},
    };
    use entity::{credentials, users};

    const TEST_ID: &str = "00000000-0000-0000-0000-000000000001";

    async fn setup_test_db() -> (Router, sea_orm::DatabaseConnection, String) {
        dotenvy::from_path("../.env").unwrap();

        // Create unique schema for this test
        let schema_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        // Connect to create schema
        let mut opt = ConnectOptions::new(dotenvy::var("TEST_DATABASE_URL").unwrap());
        opt.max_connections(10)
            .min_connections(1)
            .sqlx_logging(true);

        let db_init = Database::connect(opt)
            .await
            .expect("Connection to DB failed");

        use sea_orm::ConnectionTrait;
        db_init.execute_unprepared(&format!("CREATE SCHEMA {}", schema_name))
            .await
            .expect("Failed to create schema");

        // Reconnect with schema in search_path
        let base_url = dotenvy::var("TEST_DATABASE_URL").unwrap();
        let url_with_schema = if base_url.contains('?') {
            format!("{}&options=-c%20search_path%3D{},public", base_url, schema_name)
        } else {
            format!("{}?options=-c%20search_path%3D{},public", base_url, schema_name)
        };

        let mut opt_with_schema = ConnectOptions::new(url_with_schema);
        opt_with_schema
            .max_connections(10)
            .min_connections(1)
            .sqlx_logging(true);

        let db = Database::connect(opt_with_schema)
            .await
            .expect("Connection to DB failed");

        // Create tables in the new schema
        db.execute_unprepared(&format!(r#"
            CREATE TABLE {}.users (
                id UUID PRIMARY KEY,
                activity_id VARCHAR NOT NULL UNIQUE,
                name VARCHAR NOT NULL,
                summary VARCHAR NOT NULL,
                icon VARCHAR
            )
        "#, schema_name))
            .await
            .expect("Failed to create users table");

        db.execute_unprepared(&format!(r#"
            CREATE TABLE {}.credentials (
                user_id UUID PRIMARY KEY REFERENCES {}.users(id) ON DELETE CASCADE,
                activity_id VARCHAR NOT NULL UNIQUE,
                password_hash VARCHAR NOT NULL,
                email VARCHAR NOT NULL UNIQUE,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL
            )
        "#, schema_name, schema_name))
            .await
            .expect("Failed to create credentials table");

        // Setup test data
        let test_id = Uuid::parse_str(TEST_ID).unwrap();
        let instance_host = dotenvy::var("INSTANCE_HOST").unwrap();
        let password_hasher = Argon2PasswordHasher::new();

        // Create test user
        let user = users::ActiveModel {
            id: Set(test_id),
            activity_id: Set(format!("https://{}/users/test_user", instance_host)),
            name: Set("テスト".to_string()),
            summary: Set("".to_string()),
            icon: Set(None),
        };
        let _ = user.insert(&db).await;

        // Create test credential with hashed password
        let password_hash = password_hasher.hash("test_password").unwrap();
        let credential = credentials::ActiveModel {
            user_id: Set(test_id),
            activity_id: Set(format!("https://{}/users/test_user", instance_host)),
            password_hash: Set(password_hash.as_str().to_string()),
            email: Set("test@example.com".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };
        let _ = credential.insert(&db).await;

        let user_repository = PostgresUserRepository::new(db.clone());
        let credential_repository = PostgresCredentialRepository::new(db.clone());
        let registration_repository = PostgresUserRegistrationRepository::new(db.clone());
        let token_generator = JwtTokenGenerator::new("testtoken".to_string());
        let login_usecase = LoginUsecase::new(
            credential_repository.clone(),
            user_repository.clone(),
            password_hasher.clone(),
            token_generator.clone(),
        );
        let register_user_usecase = RegisterUserUsecase::new(
            registration_repository,
            password_hasher.clone(),
            token_generator.clone(),
        );

        // setup router: sync settings of main.app
        let router = Router::new().nest(
            "/api",
            create_user_router(login_usecase, register_user_usecase),
        );

        (router, db, schema_name)
    }

    async fn cleanup_test_db(db: &sea_orm::DatabaseConnection, schema_name: &str) {
        use sea_orm::ConnectionTrait;
        db.execute_unprepared(&format!("DROP SCHEMA {} CASCADE", schema_name))
            .await
            .expect("Failed to drop schema");
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

    #[tokio::test]
    async fn test_login_positive() {
        let (app, db, schema_name) = setup_test_db().await;

        // create request body
        let user_id = "test_user".to_string();
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

        cleanup_test_db(&db, &schema_name).await;
    }

    #[tokio::test]
    async fn test_login_invalid_user_negative() {
        let (app, db, schema_name) = setup_test_db().await;

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

        cleanup_test_db(&db, &schema_name).await;
    }

    #[tokio::test]
    async fn test_login_invalid_password_negative() {
        let (app, db, schema_name) = setup_test_db().await;

        // create request body
        let user_id = "test_user".to_string();
        let password = "invalid_password".to_string();
        let login_request = LoginRequest {
            user_id: user_id.clone(),
            password: password.clone(),
        };
        let body = serde_json::to_string(&login_request).unwrap();

        // send request
        let response = login(app, body).await;

        // validation
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        cleanup_test_db(&db, &schema_name).await;
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

    #[tokio::test]
    async fn test_register_positive() {
        let (app, db, schema_name) = setup_test_db().await;

        // create request body
        let new_user_id = "new_user";
        let new_password = "new_password";
        let new_mail_adress = "new@example.com";
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
            panic!(
                "Expected CREATED but got {:?}. Error: {}",
                status, error_msg
            );
        }
        assert_eq!(status, StatusCode::CREATED);
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        let login_response: LoginResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(new_user_id, login_response.user.acct);
        assert_eq!("テスト", login_response.user.display_name);

        cleanup_test_db(&db, &schema_name).await;
    }

    #[tokio::test]
    async fn test_register_duplicated_user_negative() {
        let (app, db, schema_name) = setup_test_db().await;

        // create request body
        let new_user_id = "test_user";
        let new_password = "new_password";
        let new_mail_adress = "new@example.com";
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

        cleanup_test_db(&db, &schema_name).await;
    }

    #[tokio::test]
    async fn test_register_duplicated_email_negative() {
        let (app, db, schema_name) = setup_test_db().await;

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
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        cleanup_test_db(&db, &schema_name).await;
    }
}
