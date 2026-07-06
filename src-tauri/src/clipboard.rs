// clipboard.rs — ZERO #[cfg(target_os)].
// Platform clipboard I/O delegated to platform::system_provider().

use crate::core::error::{op_err, AppError, AppResult};
use crate::core::fs_helper;
use crate::core::state::AppState;
use crate::core::types::{ActionKind, ClipboardInfo, FileAction, MAX_UNDO_HISTORY};
use crate::platform;
use std::path::{Path, PathBuf};
use tauri::State;

fn trim_undo(history: &mut Vec<FileAction>) {
    if history.len() > MAX_UNDO_HISTORY {
        let excess = history.len() - MAX_UNDO_HISTORY;
        history.drain(0..excess);
    }
}

pub fn copy_clipboard(state: State<AppState>, paths: Vec<String>) -> AppResult<()> {
    tracing::info!("copy_clipboard: {} items", paths.len());
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        inner.clipboard.clear();
        for p in &paths {
            inner.clipboard.push(PathBuf::from(p));
        }
        inner.clipboard_action = "copy".to_string();
    }
    let pbs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    platform::system_provider().write_clipboard(&pbs);
    Ok(())
}

pub fn cut_clipboard(state: State<AppState>, paths: Vec<String>) -> AppResult<()> {
    tracing::info!("cut_clipboard: {} items", paths.len());
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        inner.clipboard.clear();
        for p in &paths {
            inner.clipboard.push(PathBuf::from(p));
        }
        inner.clipboard_action = "cut".to_string();
    }
    let pbs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    platform::system_provider().write_clipboard(&pbs);
    Ok(())
}

pub fn paste_clipboard(state: State<AppState>, dest_dir: String) -> AppResult<()> {
    tracing::info!("paste_clipboard: to {}", dest_dir);

    if let Some(sys_paths) = platform::system_provider().read_clipboard() {
        if !sys_paths.is_empty() {
            let is_cut = state
                .inner
                .lock()
                .ok()
                .map(|i| i.clipboard_action == "cut")
                .unwrap_or(false);
            let dest = Path::new(&dest_dir);
            for src in &sys_paths {
                if !src.exists() {
                    continue;
                }
                if let Some(n) = src.file_name() {
                    let dp = fs_helper::resolve_paste_conflict(&dest.join(n));
                    if src.is_dir() {
                        fs_helper::copy_recursive(src, &dp)?;
                        if is_cut {
                            fs_helper::remove_dir_all(src)
                                .map_err(|e| op_err("Remove failed", e))?;
                        }
                    } else {
                        fs_helper::copy_file(src, &dp).map_err(|e| op_err("Copy failed", e))?;
                        if is_cut {
                            fs_helper::remove_file(src).map_err(|e| op_err("Remove failed", e))?;
                        }
                    }
                    let mut inner = state
                        .inner
                        .lock()
                        .map_err(|e| AppError::Other(e.to_string()))?;
                    inner.undo_history.push(FileAction {
                        kind: ActionKind::Copy {
                            src: src.to_string_lossy().to_string(),
                            dest: dp.to_string_lossy().to_string(),
                            was_cut: is_cut,
                        },
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    });
                    trim_undo(&mut inner.undo_history);
                }
            }
            if is_cut {
                let _ = state.inner.lock().map(|mut i| {
                    i.clipboard.clear();
                    i.clipboard_action.clear();
                });
            }
            return Ok(());
        }
    }

    let inner = state
        .inner
        .lock()
        .map_err(|e| AppError::Other(e.to_string()))?;
    if inner.clipboard.is_empty() {
        return Err(AppError::Other("Clipboard is empty".into()));
    }
    let clipboard_paths: Vec<PathBuf> = inner.clipboard.clone();
    let is_cut = inner.clipboard_action == "cut";
    drop(inner);

    let dest = Path::new(&dest_dir);
    for src_path in &clipboard_paths {
        if let Some(file_name) = src_path.file_name() {
            let dest_path = fs_helper::resolve_paste_conflict(&dest.join(file_name));
            if src_path.is_dir() {
                fs_helper::copy_recursive(src_path, &dest_path)?;
                if is_cut {
                    fs_helper::remove_dir_all(src_path)
                        .map_err(|e| op_err("Failed to remove", e))?;
                }
            } else {
                fs_helper::copy_file(src_path, &dest_path)
                    .map_err(|e| op_err("Failed to copy", e))?;
                if is_cut {
                    fs_helper::remove_file(src_path).map_err(|e| op_err("Failed to remove", e))?;
                }
            }
            if !is_cut {
                let mut inner = state
                    .inner
                    .lock()
                    .map_err(|e| AppError::Other(e.to_string()))?;
                inner.undo_history.push(FileAction {
                    kind: ActionKind::Copy {
                        src: src_path.to_string_lossy().to_string(),
                        dest: dest_path.to_string_lossy().to_string(),
                        was_cut: false,
                    },
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                });
                trim_undo(&mut inner.undo_history);
            }
        }
    }
    if is_cut {
        let _ = state.inner.lock().map(|mut i| {
            i.clipboard.clear();
            i.clipboard_action.clear();
        });
    }
    Ok(())
}

pub fn get_clipboard_info(state: State<AppState>) -> AppResult<ClipboardInfo> {
    if let Some(p) = platform::system_provider().read_clipboard() {
        if !p.is_empty() {
            return Ok(ClipboardInfo {
                paths: p.iter().map(|x| x.to_string_lossy().to_string()).collect(),
                action: "copy".into(),
            });
        }
    }
    let inner = state
        .inner
        .lock()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(ClipboardInfo {
        paths: inner
            .clipboard
            .iter()
            .map(|x| x.to_string_lossy().to_string())
            .collect(),
        action: inner.clipboard_action.clone(),
    })
}
