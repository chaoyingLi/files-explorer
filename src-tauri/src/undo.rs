use crate::error::{FsError, FsResult, op_err};
use crate::types::{ActionKind, FileAction};
use crate::state::AppState;
use crate::operations::copy_dir_recursive;
use std::fs;
use std::path::Path;
use tauri::State;

pub fn undo_last_action(state: State<AppState>) -> FsResult<String> {
    let mut history = state.undo_history.lock().map_err(|e| FsError::Other(e.to_string()))?;
    let action = history.pop().ok_or(FsError::Other("Nothing to undo".into()))?;
    match &action.kind {
        ActionKind::Delete => Err(FsError::Other("Cannot undo delete operation".into())),
        ActionKind::Rename { old_path, new_path } => {
            if !Path::new(new_path).exists() {
                return Err(FsError::Other(format!("Cannot undo: {new_path} no longer exists")));
            }
            fs::rename(new_path, old_path).map_err(|e| op_err("Undo rename failed", e))?;
            Ok(format!("Undid rename: restored {old_path}"))
        }
        ActionKind::Create { path, is_dir } => {
            if *is_dir {
                fs::remove_dir_all(path).map_err(|e| op_err("Undo create failed", e))?;
            } else {
                fs::remove_file(path).map_err(|e| op_err("Undo create failed", e))?;
            }
            Ok(format!("Undid create: removed {path}"))
        }
        ActionKind::Copy { src, dest, was_cut } => {
            let dest_path = Path::new(dest);
            if !dest_path.exists() {
                return Err(FsError::Other(format!("Cannot undo: {dest} no longer exists")));
            }
            if *was_cut {
                // Bug 11 fix: cut-paste undo — restore original file at src, then remove copy
                let src_path = Path::new(src);
                if dest_path.is_dir() {
                    copy_dir_recursive(dest_path, src_path)?;
                    fs::remove_dir_all(dest_path).map_err(|e| op_err("Undo cut failed", e))?;
                } else {
                    fs::copy(dest_path, src_path)
                        .map_err(|e| op_err("Undo cut restore failed", e))?;
                    fs::remove_file(dest_path).map_err(|e| op_err("Undo cut failed", e))?;
                }
                Ok(format!("Undid cut: restored {src}"))
            } else {
                // Regular copy: just remove the copy
                if dest_path.is_dir() {
                    fs::remove_dir_all(dest_path)
                        .map_err(|e| op_err("Undo copy failed", e))?;
                } else {
                    fs::remove_file(dest_path).map_err(|e| op_err("Undo copy failed", e))?;
                }
                Ok(format!("Undid copy of {src}: removed {dest}"))
            }
        }
    }
}

pub fn get_undo_info(state: State<AppState>) -> FsResult<Option<FileAction>> {
    let history = state.undo_history.lock().map_err(|e| FsError::Other(e.to_string()))?;
    Ok(history.last().cloned())
}
