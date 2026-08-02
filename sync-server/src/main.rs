use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

use aether_sync_server::{handlers, storage::Storage};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        return run_healthcheck();
    }

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
    if server_seed_phrase.trim().len() < 12 {
        return Err("SERVER_SEED_PHRASE must be at least 12 non-whitespace characters".into());
    }
    let (broadcast_tx, _) = broadcast::channel::<String>(16);

    let app = handlers::router(
        storage,
        broadcast_tx,
        Arc::from(server_seed_phrase.into_boxed_str()),
    );
    let addr: SocketAddr = ([0, 0, 0, 0], 8080).into();
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

fn run_healthcheck() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let address: SocketAddr = "127.0.0.1:8080".parse()?;
    let mut stream = TcpStream::connect_timeout(&address, std::time::Duration::from_secs(2))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
    stream.write_all(b"GET /ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut response = [0; 128];
    let bytes_read = stream.read(&mut response)?;
    let response = std::str::from_utf8(&response[..bytes_read])?;
    if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
        return Ok(());
    }

    Err(format!("readiness endpoint returned unexpected response: {response:?}").into())
}
