//! Sync wire types: envelopes, push/pull payloads.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEnvelope {
    pub entity: String,
    pub id: String,
    pub op: ChangeOp,
    pub data: Option<serde_json::Value>,
    pub updated_at: i64,
    pub device_id: String,
    pub device_hostname: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeOp {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<EncryptedChange>,
    pub timestamp: i64,
    pub has_more: bool,
}
