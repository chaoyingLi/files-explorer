use crate::error::{FsError, FsResult, op_err};
use crate::types::{ActionKind, FileAction, PASTE_CONFLICT_SUFFIX, MAX_UNDO_HISTORY};
use crate::state::AppState;
use log;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

/// Trim undo history to MAX_UNDO_HISTORY entries
pub fn trim_undo_history(history: &mut Vec<FileAction>) {
    if history.len() > MAX_UNDO_HISTORY {
        let excess = history.len() - MAX_UNDO_HISTORY;
        history.drain(0..excess);
    }
}

pub fn get_parent_directory(path: String) -> FsResult<String> {
    let path = Path::new(&path);
    match path.parent() {
        Some(parent) => Ok(parent.to_string_lossy().to_string()),
        None => Err(FsError::InvalidPath("No parent directory".into())),
    }
}

pub fn create_directory(state: State<AppState>, path: String) -> FsResult<()> {
    fs::create_dir_all(&path).map_err(|e| op_err("Failed to create directory", e))?;
    let mut history = state.undo_history.lock().map_err(|e| FsError::Other(e.to_string()))?;
    history.push(FileAction {
        kind: ActionKind::Create {
            path: path.clone(),
            is_dir: true,
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    trim_undo_history(&mut history);
    Ok(())
}

pub fn create_file(state: State<AppState>, path: String) -> FsResult<()> {
    if Path::new(&path).exists() {
        return Err(FsError::AlreadyExists("File already exists".into()));
    }
    fs::write(&path, "").map_err(|e| op_err("Failed to create file", e))?;
    let mut history = state.undo_history.lock().map_err(|e| FsError::Other(e.to_string()))?;
    history.push(FileAction {
        kind: ActionKind::Create {
            path: path.clone(),
            is_dir: false,
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    trim_undo_history(&mut history);
    Ok(())
}

pub fn delete_item(path: String, permanently: bool) -> FsResult<()> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err(FsError::NotFound("Path does not exist".into()));
    }

    if permanently {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| op_err("Failed to delete", e))
        } else {
            fs::remove_file(path).map_err(|e| op_err("Failed to delete", e))
        }
    } else {
        // Send to trash
        trash::delete(path).map_err(|e| op_err("Failed to move to trash", e))
    }
}

pub fn rename_item(state: State<AppState>, old_path: String, new_path: String) -> FsResult<()> {
    fs::rename(&old_path, &new_path).map_err(|e| op_err("Failed to rename", e))?;
    let mut history = state.undo_history.lock().map_err(|e| FsError::Other(e.to_string()))?;
    history.push(FileAction {
        kind: ActionKind::Rename {
            old_path: old_path.clone(),
            new_path: new_path.clone(),
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    trim_undo_history(&mut history);
    Ok(())
}

pub fn move_files(paths: Vec<String>, dest_dir: String, copy: bool) -> FsResult<()> {
    log::info!(
        "move_files: {} items to {} (copy={})",
        paths.len(),
        dest_dir,
        copy
    );
    let dest = Path::new(&dest_dir);
    for src_str in &paths {
        let src = Path::new(src_str);
        if !src.exists() {
            continue;
        }
        let file_name = match src.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        let dest_path = dest.join(&file_name);
        if dest_path.exists() {
            return Err(FsError::AlreadyExists(format!("Target already exists: {}", file_name)));
        }
        // Try fast rename first (same filesystem)
        if !copy && std::fs::rename(src, &dest_path).is_ok() {
            continue;
        }
        // Cross-device or copy: fall back to copy-then-delete
        if src.is_dir() {
            copy_dir_recursive(src, &dest_path)?;
        } else {
            std::fs::copy(src, &dest_path).map_err(|e| op_err("Failed to copy", e))?;
        }
        if !copy {
            if src.is_dir() {
                std::fs::remove_dir_all(src)
                    .map_err(|e| op_err("Failed to remove source", e))?;
            } else {
                std::fs::remove_file(src).map_err(|e| op_err("Failed to remove source", e))?;
            }
        }
    }
    Ok(())
}

/// Recursively copy a directory and its contents.
pub fn copy_dir_recursive(src: &Path, dest: &Path) -> FsResult<()> {
    fs::create_dir_all(dest).map_err(|e| op_err("Failed to create directory", e))?;

    for entry in fs::read_dir(src).map_err(|e| op_err("Failed to read directory", e))? {
        let entry = entry.map_err(|e| op_err("Failed to read entry", e))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| op_err("Failed to copy file", e))?;
        }
    }

    Ok(())
}

/// Resolve a naming conflict when pasting a file by appending a suffix or counter.
pub fn resolve_paste_conflict(path: &Path) -> PathBuf {
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

    let mut counter: u32 = 1;
    loop {
        let new_name = if counter == 1 {
            if ext.is_empty() {
                format!("{}{}", stem, PASTE_CONFLICT_SUFFIX)
            } else {
                format!("{}{}.{}", stem, PASTE_CONFLICT_SUFFIX, ext)
            }
        } else {
            if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            }
        };
        let candidate = parent.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
        if counter > 999 {
            break;
        }
    }
    // Fallback: use timestamp
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    parent.join(format!("{}_{}.{}", stem, ts, ext))
}
