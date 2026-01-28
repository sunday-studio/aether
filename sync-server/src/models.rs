use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device_id: String,
    pub device_hostname: String,
    pub changes: Vec<EncryptedChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedChange {
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Serialize)]
pub struct PullResponse {
    pub changes: Vec<EncryptedChange>,
    pub timestamp: i64,
    pub has_more: bool,
}
