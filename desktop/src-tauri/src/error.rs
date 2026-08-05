use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("LibSQL error: {0}")]
    LibSQL(#[from] libsql::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::utils::performance_ledger::record_error(
            "tauri.command",
            "Tauri command returned an error",
            serde_json::json!({
                "kind": self.kind(),
                "error": self.to_string(),
            }),
        );
        // Serialize as a simple string message for Tauri IPC
        serializer.serialize_str(&self.to_string())
    }
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            Self::LibSQL(_) => "libsql",
            Self::Serialization(_) => "serialization",
            Self::BadRequest(_) => "bad_request",
            Self::NotFound(_) => "not_found",
            Self::Internal(_) => "internal",
            Self::Io(_) => "io",
            Self::ModelError(_) => "model",
            Self::EncryptionError(_) => "encryption",
            Self::ProviderNotConfigured(_) => "provider_not_configured",
            Self::Sync(_) => "sync",
            Self::Http(_) => "http",
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
