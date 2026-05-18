use std::sync::LazyLock;

pub mod apply;
pub mod encryption;
pub mod engine;
pub mod media;
pub mod metadata;
pub mod outbox;
pub mod pull;
pub mod push;
pub mod register;
pub mod scheduler;
pub mod types;
pub mod ws;

pub use engine::{SyncEngine, SyncStatus};
pub use types::{ChangeEnvelope, ChangeOp, EncryptedChange, PullCursor, PullResponse, PushRequest};

const SYNC_HTTP_CONNECT_TIMEOUT_SECS: u64 = 5;
const SYNC_HTTP_REQUEST_TIMEOUT_SECS: u64 = 15;

static SYNC_HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(
            SYNC_HTTP_CONNECT_TIMEOUT_SECS,
        ))
        .timeout(std::time::Duration::from_secs(
            SYNC_HTTP_REQUEST_TIMEOUT_SECS,
        ))
        .build()
        .expect("sync HTTP client should build")
});

fn http_client() -> &'static reqwest::Client {
    &SYNC_HTTP_CLIENT
}

fn authenticated_request(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    device_id: &str,
    device_token: &str,
) -> reqwest::RequestBuilder {
    client
        .request(method, url)
        .header("x-aether-device-id", device_id)
        .bearer_auth(device_token)
}
