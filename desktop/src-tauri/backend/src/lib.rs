pub mod api;
pub mod db;
pub mod error;
pub mod handlers;
pub mod utils;

pub use db::DbState;
pub use error::{AppError, Result};

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// Start the backend server in a background task
/// This function initializes the database, runs migrations, and starts the HTTP server
pub async fn start_server() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing (only if not already initialized)
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aether_backend=info,tower_http=debug".into()),
        )
        .try_init();

    // Initialize database
    let db_state = db::initialize().await?;
    tracing::info!("Database initialized successfully");

    // Run migrations
    let database = db::connection::get_database(&db_state);
    db::migrations::run_migrations(&database).await?;
    tracing::info!("Migrations completed");

    // Build application with routes
    let app = create_app(db_state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "9119".to_string())
        .parse::<u16>()
        .unwrap_or(9119);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Aether backend running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| AppError::Io(e))?;
    axum::serve(listener, app).await
        .map_err(|e| AppError::Io(e))?;

    Ok(())
}

fn create_app(state: db::DbState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(api::register_routes(state))
        .layer(cors)
}