// core/error.rs
// Unified cross-platform error type with thiserror derive.
// All IO / system / business / Tauri errors converge here.
// Auto-logged when returned from #[tauri::command] via the global tracing subscriber.

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    /// The path or resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Insufficient permissions.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Target already exists.
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// General I/O error.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Malformed or invalid path.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Tauri / IPC error.
    #[error("Tauri error: {0}")]
    Tauri(String),

    /// Serialisation error.
    #[error("Serialize error: {0}")]
    Serialize(String),

    /// Catch-all.
    #[error("{0}")]
    Other(String),
}

impl AppError {
    /// Log this error via tracing and return it (for use in `map_err` chains).
    #[track_caller]
    pub fn log(self) -> Self {
        let loc = std::panic::Location::caller();
        tracing::error!(
            error = %self,
            file = loc.file(),
            line = loc.line(),
            module = module_path!(),
            "AppError"
        );
        self
    }
}

// ── From impls ──

impl From<std::io::Error> for AppError {
    #[track_caller]
    fn from(err: std::io::Error) -> Self {
        let e = match err.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(err.to_string()),
            std::io::ErrorKind::AlreadyExists => AppError::AlreadyExists(err.to_string()),
            _ => AppError::IoError(err.to_string()),
        };
        e.log()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialize(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Other(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Other(s.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        AppError::Tauri(err.to_string())
    }
}

/// Convenience: wrap an operation name and error message.
#[track_caller]
pub fn op_err(op: &str, e: impl std::fmt::Display) -> AppError {
    AppError::IoError(format!("{op}: {e}")).log()
}

pub type AppResult<T> = Result<T, AppError>;

// Backward-compatible aliases
pub type FsError = AppError;
pub type FsResult<T> = AppResult<T>;
