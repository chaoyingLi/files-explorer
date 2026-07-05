// core/fs_helper.rs
// Unified cross-platform file-system helper.
// All code in core/, commands/, and the rest of the app MUST route
// file operations through this module — NEVER call std::fs directly
// for metadata, hidden checks, or shortcut resolution.

use crate::core::error::AppError;
use crate::core::types::FileEntry;
use crate::platform;
use std::fs;
use std::path::{Path, PathBuf};

/// Read a directory into `FileEntry` items with platform-normalised metadata.
/// If `skip_hidden` is true, files considered "hidden" by the platform are omitted.
pub fn list_directory(path: &Path, skip_hidden: bool) -> Result<Vec<FileEntry>, AppError> {
    if !path.exists() {
        return Err(AppError::NotFound(format!("Path not found: {}", path.display())));
    }
    if !path.is_dir() {
        return Err(AppError::InvalidPath(format!("Not a directory: {}", path.display())));
    }

    let fs_ext = platform::fs_ext_provider();
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)
        .map_err(|e| AppError::IoError(format!("Failed to read directory: {}", e)))?
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_path = entry.path();

        if skip_hidden && fs_ext.is_hidden(&file_path) {
            continue;
        }

        match file_entry_from_path_inner(&file_path, &file_path) {
            Ok(fe) => entries.push(fe),
            Err(_) => continue,
        }
    }

    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

/// Build a `FileEntry` from a path, using platform-normalised metadata.
pub fn file_entry_from_path(path: &Path) -> Result<FileEntry, AppError> {
    file_entry_from_path_inner(path, path)
}

/// Resolve shortcut target if applicable, then build a `FileEntry`.
fn file_entry_from_path_inner(display_path: &Path, fs_path: &Path) -> Result<FileEntry, AppError> {
    let fs_ext = platform::fs_ext_provider();
    let metadata = fs::metadata(fs_path).map_err(AppError::from)?;
    let is_dir = metadata.is_dir();
    let size = if is_dir { 0 } else { metadata.len() };
    let modified = fs_ext
        .modified_time(&metadata)
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let created = fs_ext
        .created_time(&metadata)
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let name = display_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = display_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(FileEntry {
        name,
        path: path_for_frontend(display_path),
        is_dir,
        size,
        modified,
        created,
        extension,
    })
}

/// Resolve a path: if it's a shortcut/alias/symlink, return the target.
pub fn resolve_path(path: &Path) -> PathBuf {
    platform::fs_ext_provider()
        .resolve_shortcut(path)
        .unwrap_or_else(|| path.to_path_buf())
}

/// Check if a path exists and is accessible.
pub fn check_access(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Err(AppError::NotFound(format!("Path not found: {}", path.display())));
    }
    fs::metadata(path).map_err(AppError::from)?;
    Ok(())
}

/// Check if a path is writable, with platform-specific hints.
pub fn check_writable(path: &Path) -> Result<(), AppError> {
    platform::fs_ext_provider().check_writable(path)
}

/// Determine whether a path is hidden by platform convention.
pub fn is_hidden(path: &Path) -> bool {
    platform::fs_ext_provider().is_hidden(path)
}

/// Determine whether a path is a symlink/shortcut.
pub fn is_symlink(path: &Path) -> bool {
    platform::fs_ext_provider().is_symlink(path)
}

/// Normalise a path for frontend consumption.
/// On Windows, backslashes are converted to forward slashes.
pub fn path_for_frontend(path: &Path) -> String {
    platform::fs_ext_provider().normalize_frontend_path(path)
}

/// Case-insensitive filename comparison per platform FS semantics.
pub fn filename_eq(a: &str, b: &str) -> bool {
    platform::fs_ext_provider().path_eq(Path::new(a), Path::new(b))
}

/// Read file contents with a size limit.
pub fn read_file_limited(path: &Path, max_bytes: usize) -> Result<Vec<u8>, AppError> {
    let metadata = fs::metadata(path).map_err(AppError::from)?;
    if metadata.len() > max_bytes as u64 {
        return Err(AppError::Other(format!(
            "File too large: {} bytes (max {})",
            metadata.len(),
            max_bytes
        )));
    }
    fs::read(path).map_err(AppError::from)
}

/// Ensure all parent directories of `path` exist.
pub fn ensure_parent(path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::from)?;
    }
    Ok(())
}

/// Recursively copy a directory or file.
pub fn copy_recursive(src: &Path, dest: &Path) -> Result<(), AppError> {
    if src.is_dir() {
        fs::create_dir_all(dest)
            .map_err(|e| AppError::IoError(format!("mkdir: {}", e)))?;
        for entry in
            fs::read_dir(src).map_err(|e| AppError::IoError(format!("read_dir: {}", e)))?
        {
            let entry =
                entry.map_err(|e| AppError::IoError(format!("entry: {}", e)))?;
            copy_recursive(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else {
        fs::copy(src, dest)
            .map_err(|e| AppError::IoError(format!("copy: {}", e)))?;
    }
    Ok(())
}

/// Resolve a naming conflict by appending a suffix or number.
pub fn resolve_paste_conflict(path: &Path) -> PathBuf {
    use crate::core::types::PASTE_CONFLICT_SUFFIX;
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    for counter in 1u32..1000 {
        let new_name = if counter == 1 {
            if ext.is_empty() {
                format!("{}{}", stem, PASTE_CONFLICT_SUFFIX)
            } else {
                format!("{}{}.{}", stem, PASTE_CONFLICT_SUFFIX, ext)
            }
        } else if ext.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, ext)
        };
        let candidate = parent.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    parent.join(format!("{}_{}.{}", stem, ts, ext))
}

// Backward-compatible aliases
pub use copy_recursive as copy_dir_recursive;
