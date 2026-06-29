use crate::types::{ts_from_metadata, FileEntry, LIST_BATCH_SIZE};
use log;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        log::warn!("list_directory: path not found: {}", path);
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();

    match fs::read_dir(&path) {
        Ok(read_dir) => {
            for entry in read_dir.flatten() {
                let file_path = entry.path();
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => match fs::metadata(&file_path) {
                        Ok(m) => m,
                        Err(_) => continue,
                    },
                };

                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = metadata.is_dir();
                let size = if is_dir { 0 } else { metadata.len() };
                let modified = ts_from_metadata(&metadata, fs::Metadata::modified);
                let created = ts_from_metadata(&metadata, fs::Metadata::created);
                let extension = file_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                entries.push(FileEntry {
                    name,
                    path: file_path.to_string_lossy().to_string(),
                    is_dir,
                    size,
                    modified,
                    created,
                    extension,
                });
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }

    // Sort: directories first, then alphabetical
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

/// List directory contents with streaming via events.
/// The generation counter is incremented but NOT used for stale-thread cancellation
/// to avoid multi-pane interference during session restore. The frontend's
/// `navigateSeq` already filters stale responses.
pub fn list_directory_streamed(
    app: AppHandle,
    path: String,
    navigate_gen: Arc<AtomicU64>,
) -> Result<(), String> {
    let dir = Path::new(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    navigate_gen.fetch_add(1, Ordering::SeqCst);
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut batch: Vec<FileEntry> = Vec::new();
        if let Ok(rd) = fs::read_dir(&path) {
            for e in rd.flatten() {
                let fp = e.path();
                let md = e.metadata().ok().or_else(|| fs::metadata(&fp).ok());
                let Some(md) = md else { continue };
                batch.push(FileEntry {
                    name: e.file_name().to_string_lossy().to_string(),
                    path: fp.to_string_lossy().to_string(),
                    is_dir: md.is_dir(),
                    size: if md.is_dir() { 0 } else { md.len() },
                    modified: ts_from_metadata(&md, fs::Metadata::modified),
                    created: ts_from_metadata(&md, fs::Metadata::created),
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
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }

    let metadata = fs::metadata(&p).map_err(|e| format!("Failed to get metadata: {}", e))?;

    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let is_dir = metadata.is_dir();
    let size = if is_dir { 0 } else { metadata.len() };
    let modified = ts_from_metadata(&metadata, fs::Metadata::modified);
    let created = ts_from_metadata(&metadata, fs::Metadata::created);
    let extension = p
        .extension()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(FileEntry {
        name,
        path: p.to_string_lossy().to_string(),
        is_dir,
        size,
        modified,
        created,
        extension,
    })
}

pub fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}
