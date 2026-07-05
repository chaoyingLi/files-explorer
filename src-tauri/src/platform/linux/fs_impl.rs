// platform/linux/fs_impl.rs
// Linux file-system extension implementation.

use crate::core::error::AppError;
use crate::platform::PlatformFsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct FsExtImpl;

impl PlatformFsExt for FsExtImpl {
    fn resolve_shortcut(&self, path: &Path) -> Option<PathBuf> {
        // Try symlink resolution first.
        std::fs::read_link(path).ok()
    }

    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
    }

    fn created_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        // Linux ext4/XFS: birth time may not be available.
        // Fall back to created() which returns ctime on many filesystems.
        metadata.created().ok()
    }

    fn modified_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        metadata.modified().ok()
    }

    fn accessed_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        metadata.accessed().ok()
    }

    fn is_symlink(&self, path: &Path) -> bool {
        path.is_symlink()
    }

    fn path_eq(&self, a: &Path, b: &Path) -> bool {
        // Linux file systems are case-sensitive by default.
        a == b
    }

    fn normalize_frontend_path(&self, path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn check_writable(&self, path: &Path) -> Result<(), AppError> {
        match path.metadata() {
            Ok(meta) => {
                let mode = meta.permissions().mode();
                if mode & 0o200 == 0 {
                    return Err(AppError::PermissionDenied(
                        "No write permission. Use chmod or run with elevated privileges.".into(),
                    ));
                }
                Ok(())
            }
            Err(_) => {
                // Check parent directory writability
                if let Some(parent) = path.parent() {
                    return self.check_writable(parent);
                }
                Err(AppError::PermissionDenied(
                    "Cannot access path. Check directory permissions.".into(),
                ))
            }
        }
    }

    fn request_write_permission(&self, _path: &Path) -> Result<(), AppError> {
        // Linux: no OS-level permission escalation mechanism.
        // User must chmod or run with sudo.
        Ok(())
    }
}

impl FsExtImpl {
    pub fn instance() -> &'static FsExtImpl {
        &FsExtImpl
    }
}
