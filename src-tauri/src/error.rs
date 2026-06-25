use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum FsError {
    NotFound(String),
    PermissionDenied(String),
    AlreadyExists(String),
    IoError(String),
    InvalidPath(String),
    Other(String),
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::NotFound(msg) => write!(f, "Not found: {}", msg),
            FsError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            FsError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            FsError::IoError(msg) => write!(f, "I/O error: {}", msg),
            FsError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            FsError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<std::io::Error> for FsError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => FsError::NotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => FsError::PermissionDenied(err.to_string()),
            std::io::ErrorKind::AlreadyExists => FsError::AlreadyExists(err.to_string()),
            _ => FsError::IoError(err.to_string()),
        }
    }
}

impl From<String> for FsError {
    fn from(s: String) -> Self {
        FsError::Other(s)
    }
}

impl From<&str> for FsError {
    fn from(s: &str) -> Self {
        FsError::Other(s.to_string())
    }
}

/// Convenience helper to wrap an operation name and error into an IoError variant
pub fn op_err(op: &str, e: impl std::fmt::Display) -> FsError {
    FsError::IoError(format!("{op}: {e}"))
}

// Convenience: Result alias used across commands
pub type FsResult<T> = Result<T, FsError>;
