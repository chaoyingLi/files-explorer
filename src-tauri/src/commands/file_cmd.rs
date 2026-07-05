// commands/file_cmd.rs
// File-related Tauri commands.

use crate::core::error::{op_err, AppError, AppResult};
use crate::core::fs_helper;
use crate::core::state::AppState;
use crate::core::types::{
    ActionKind, FileAction, FileEntry, LIST_BATCH_SIZE, MAX_UNDO_HISTORY,
};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};

/// Trim undo history to MAX_UNDO_HISTORY.
fn trim_undo(history: &mut Vec<FileAction>) {
    if history.len() > MAX_UNDO_HISTORY {
        let excess = history.len() - MAX_UNDO_HISTORY;
        history.drain(0..excess);
    }
}


pub fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let p = Path::new(&path);
    fs_helper::list_directory(p, false).map_err(|e| e.to_string())
}


pub fn list_directory_streamed(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    let dir = Path::new(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    state.navigate_gen.fetch_add(1, Ordering::SeqCst);
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut batch: Vec<FileEntry> = Vec::new();
        if let Ok(rd) = fs::read_dir(&path) {
            for e in rd.flatten() {
                let fp = e.path();
                let md = e.metadata().ok().or_else(|| fs::metadata(&fp).ok());
                let Some(md) = md else { continue };
                let fs_ext = crate::platform::fs_ext_provider();
                let modified = fs_ext
                    .modified_time(&md)
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let created = fs_ext
                    .created_time(&md)
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                batch.push(FileEntry {
                    name: e.file_name().to_string_lossy().to_string(),
                    path: fs_helper::path_for_frontend(&fp),
                    is_dir: md.is_dir(),
                    size: if md.is_dir() { 0 } else { md.len() },
                    modified,
                    created,
                    extension: fp
                        .extension()
                        .map(|x| x.to_string_lossy().to_string())
                        .unwrap_or_default(),
                });
                if batch.len() >= LIST_BATCH_SIZE {
                    let _ = app2.emit("list-progress", std::mem::take(&mut batch));
                }
            }
        }
        if !batch.is_empty() {
            let _ = app2.emit("list-progress", batch);
        }
        let _ = app2.emit("list-done", true);
    });
    Ok(())
}


pub fn get_file_info(path: String) -> Result<FileEntry, String> {
    let p = Path::new(&path);
    fs_helper::file_entry_from_path(p).map_err(|e| e.to_string())
}


pub fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}


pub fn get_parent_directory(path: String) -> Result<String, AppError> {
    let p = Path::new(&path);
    match p.parent() {
        Some(parent) => Ok(parent.to_string_lossy().to_string()),
        None => Err(AppError::InvalidPath("No parent directory".into())),
    }
}


pub fn create_directory(state: State<AppState>, path: String) -> AppResult<()> {
    fs::create_dir_all(&path).map_err(|e| op_err("Failed to create directory", e))?;
    let mut inner = state.inner.lock().map_err(|e| AppError::Other(e.to_string()))?;
    inner.undo_history.push(FileAction {
        kind: ActionKind::Create { path: path.clone(), is_dir: true },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    trim_undo(&mut inner.undo_history);
    Ok(())
}


pub fn create_file(state: State<AppState>, path: String) -> AppResult<()> {
    if Path::new(&path).exists() {
        return Err(AppError::AlreadyExists("File already exists".into()));
    }
    fs::write(&path, "").map_err(|e| op_err("Failed to create file", e))?;
    let mut inner = state.inner.lock().map_err(|e| AppError::Other(e.to_string()))?;
    inner.undo_history.push(FileAction {
        kind: ActionKind::Create { path: path.clone(), is_dir: false },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    trim_undo(&mut inner.undo_history);
    Ok(())
}


pub fn delete_item(path: String, permanently: bool) -> AppResult<()> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound("Path does not exist".into()));
    }
    if permanently {
        if p.is_dir() {
            fs::remove_dir_all(p).map_err(|e| op_err("Failed to delete", e))
        } else {
            fs::remove_file(p).map_err(|e| op_err("Failed to delete", e))
        }
    } else {
        trash::delete(p).map_err(|e| op_err("Failed to move to trash", e))
    }
}


pub fn rename_item(state: State<AppState>, old_path: String, new_path: String) -> AppResult<()> {
    fs::rename(&old_path, &new_path).map_err(|e| op_err("Failed to rename", e))?;
    let mut inner = state.inner.lock().map_err(|e| AppError::Other(e.to_string()))?;
    inner.undo_history.push(FileAction {
        kind: ActionKind::Rename { old_path, new_path },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    trim_undo(&mut inner.undo_history);
    Ok(())
}


pub fn move_files(paths: Vec<String>, dest_dir: String, copy: bool) -> AppResult<()> {
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
            return Err(AppError::AlreadyExists(format!("Target already exists: {}", file_name)));
        }
        if !copy && fs::rename(src, &dest_path).is_ok() {
            continue;
        }
        fs_helper::copy_recursive(src, &dest_path)?;
        if !copy {
            if src.is_dir() {
                fs::remove_dir_all(src).map_err(|e| op_err("Failed to remove source", e))?;
            } else {
                fs::remove_file(src).map_err(|e| op_err("Failed to remove source", e))?;
            }
        }
    }
    Ok(())
}
