// platform/fs_ext.rs
// Platform-specific file-system extensions.
// Covers: shortcuts/symlinks/aliases, hidden files, permissions,
//   metadata normalisation, path comparison, frontend-path serialisation.

use crate::core::error::AppError;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub trait PlatformFsExt: Send + Sync {
    // ── Shortcut / symlink / alias resolution ──

    /// Resolve a platform shortcut to its target.
    ///
    /// | Platform | Handles |
    /// |----------|---------|
    /// | Windows  | `.lnk` files via IShellLink |
    /// | macOS    | aliases via CFURL bookmark + plain symlinks |
    /// | Linux    | symlinks via `read_link` + `.desktop` files |
    ///
    /// Returns `None` when the path is not a shortcut or resolution fails.
    fn resolve_shortcut(&self, path: &Path) -> Option<PathBuf>;

    // ── Hidden file detection ──

    /// Whether a file is considered "hidden" by the platform.
    ///
    /// | Platform | Rule |
    /// |----------|------|
    /// | Windows  | `FILE_ATTRIBUTE_HIDDEN` or dot-prefix fallback |
    /// | macOS    | dot-prefix or `UF_HIDDEN` flag |
    /// | Linux    | dot-prefix |
    fn is_hidden(&self, path: &Path) -> bool;

    // ── Metadata normalisation ──

    /// Creation / birth time, normalised across platforms.
    ///
    /// macOS uses `st_birthtime`; Linux falls back to `created()` (which may
    /// return `Ok` but be the ctime on some filesystems).
    fn created_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime>;

    /// Last modification time.
    fn modified_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime>;

    /// Last access time.
    fn accessed_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime>;

    // ── Symlink detection ──

    fn is_symlink(&self, path: &Path) -> bool;

    // ── Path comparison ──

    /// Compare two paths, respecting the platform's default case sensitivity.
    ///
    /// | Platform | Behaviour |
    /// |----------|-----------|
    /// | Windows  | Case-insensitive (NTFS default) |
    /// | macOS    | Case-insensitive (HFS+/APFS default) |
    /// | Linux    | Case-sensitive (ext4/XFS default) |
    fn path_eq(&self, a: &Path, b: &Path) -> bool;

    // ── Frontend path serialisation ──

    /// Normalise path separators for frontend consumption.
    /// On Windows this converts backslashes to forward slashes.
    fn normalize_frontend_path(&self, path: &Path) -> String;

    // ── Permission checks ──

    /// Check whether the current process can write to the given path.
    /// Returns a human-readable remediation hint in the error message.
    fn check_writable(&self, path: &Path) -> Result<(), AppError>;

    /// Best-effort request for elevated write permission.
    ///
    /// | Platform | Mechanism |
    /// |----------|-----------|
    /// | macOS    | Security-scoped bookmark / `NSOpenPanel` |
    /// | Windows  | (reserved — currently no-op) |
    /// | Linux    | (reserved — currently no-op) |
    fn request_write_permission(&self, _path: &Path) -> Result<(), AppError> {
        Ok(())
    }
}
