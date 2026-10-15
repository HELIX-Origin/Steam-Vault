//! Unified error type for the Steam Vault backend.
//!
//! Every fallible operation returns [`AppResult`]. The error implements
//! [`serde::Serialize`] so it can be surfaced directly to the Svelte frontend
//! through Tauri command results.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse VDF: {0}")]
    Vdf(String),

    #[error("failed to parse binary VDF: {0}")]
    BinVdf(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("Steam appears to be running: {0}")]
    SteamRunning(String),

    #[error("invalid data: {0}")]
    Invalid(String),

    #[error("{0}")]
    Message(String),
}

impl AppError {
    pub fn msg(message: impl Into<String>) -> Self {
        AppError::Message(message.into())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
