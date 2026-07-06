// platform/macos/fs_impl.rs
// macOS file-system extension implementation.

use crate::core::error::AppError;
use crate::platform::PlatformFsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct FsExtImpl;

impl PlatformFsExt for FsExtImpl {
    fn resolve_shortcut(&self, path: &Path) -> Option<PathBuf> {
        // macOS: plain symlink via read_link; aliases require
        // CFURLCreateByResolvingBookmarkData — deferred.
        std::fs::read_link(path).ok().or_else(|| {
            // Heuristic: if extension is empty or path is an alias, try
            // reading the resource fork (shortcut marker).
            // For now, just return None.
            None
        })
    }

    fn is_hidden(&self, path: &Path) -> bool {
        // Rule 1: dot-prefix name
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            return true;
        }
        // Rule 2: UF_HIDDEN flag on macOS
        {
            use std::os::darwin::fs::MetadataExt;
            if let Ok(meta) = path.metadata() {
                const UF_HIDDEN: u32 = 0x8000;
                if meta.st_flags() & UF_HIDDEN != 0 {
                    return true;
                }
            }
        }
        false
    }

    fn created_time(&self, metadata: &std::fs::Metadata) -> Option<SystemTime> {
        {
            use std::os::macos::fs::MetadataExt;
            let bt = metadata.st_birthtime();
            if bt >= 0 {
                return Some(
                    UNIX_EPOCH
                        + Duration::from_secs(bt as u64)
                        + Duration::from_nanos(metadata.st_birthtime_nsec() as u64),
                );
            }
        }
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
        // HFS+ and APFS default to case-insensitive comparison.
        a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
    }

    fn normalize_frontend_path(&self, path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn check_writable(&self, path: &Path) -> Result<(), AppError> {
        if path.exists() {
            let meta = std::fs::metadata(path).map_err(|e| AppError::IoError(e.to_string()))?;
            let perm = meta.permissions();
            if perm.readonly() {
                return Err(AppError::PermissionDenied(
                    "File is read-only. macOS sandbox may also restrict access.".into(),
                ));
            }
        }
        // Test actual writability by attempting to create a temp file
        let test = if path.is_dir() {
            path.join(".write_test")
        } else {
            path.with_extension("write_test_tmp")
        };
        match std::fs::write(&test, b"") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test);
                Ok(())
            }
            Err(_) => Err(AppError::PermissionDenied(
                "No write permission. Grant access via System Preferences → Privacy & Security."
                    .into(),
            )),
        }
    }

    fn request_write_permission(&self, _path: &Path) -> Result<(), AppError> {
        // macOS: can trigger NSOpenPanel with security-scoped bookmark.
        // Deferred — requires frontend integration.
        Ok(())
    }
}

impl FsExtImpl {
    pub fn instance() -> &'static FsExtImpl {
        &FsExtImpl
    }
}
