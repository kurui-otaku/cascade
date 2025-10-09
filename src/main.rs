use axum::{Router, routing::get, serve};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = app().await?;
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}

async fn app() -> Result<Router, Box<dyn std::error::Error>> {
    dotenvy::from_path("../.env").expect(".env file not found");
    let mut opt = ConnectOptions::new(dotenvy::var("DATABASE_URL")?);
    opt.max_connections(10)
        .min_connections(1)
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    let state = AppState { db };
    let app = Router::new()
        .route("/", get(|| async { "Hello, Axum!" }))
        .with_state(state);
    Ok(app)
}

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}
