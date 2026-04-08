use std::path::PathBuf;
use std::sync::Arc;

use aether_sync_server::{handlers, storage::Storage};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let data_root = std::env::var("DATA_ROOT").unwrap_or_else(|_| "./data".into());
    let data_root = PathBuf::from(&data_root);
    let db_path = data_root.join("sync.db");

    let storage = Storage::new(&db_path, &data_root)?;
    storage.initialize_salt()?;
    let storage = Arc::new(storage);
    let server_seed_phrase = std::env::var("SERVER_SEED_PHRASE")
        .or_else(|_| std::env::var("SERVER_PASSPHRASE"))
        .map_err(|_| "SERVER_SEED_PHRASE is required")?;
    let (broadcast_tx, _) = broadcast::channel::<String>(16);

    let app = handlers::router(
        storage,
        broadcast_tx,
        Arc::from(server_seed_phrase.into_boxed_str()),
    );
    let addr: std::net::SocketAddr = ([0, 0, 0, 0], 8080).into();
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
