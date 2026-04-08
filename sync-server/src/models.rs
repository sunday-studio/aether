use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub device_id: String,
    pub hostname: Option<String>,
    pub server_seed_phrase: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub device_token: String,
    pub salt: String,
}

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub batch_id: String,
    pub device_hostname: String,
    pub changes: Vec<EncryptedChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedChange {
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullCursor {
    pub received_at: i64,
    pub change_id: i64,
}

#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub changes: Vec<EncryptedChange>,
    pub next_cursor: Option<PullCursor>,
    pub has_more: bool,
}
