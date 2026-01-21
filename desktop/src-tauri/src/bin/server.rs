use crate::{db, error::{AppError, Result}};
use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "Aether=info,tower_http=debug".into()),
        )
        .init();

    // Initialize database
    let db_state = db::initialize().await?;
    tracing::info!("Database initialized successfully");

    // Run migrations
    let database = crate::db::connection::get_database(&db_state);
    crate::db::migrations::run_migrations(&database).await?;
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

    // Import swagger_ui from the api module
    use crate::api::openapi;
    
    Router::new()
        .merge(crate::api::register_routes(state))
        // .merge(openapi::swagger_ui())
        .layer(cors)
}
