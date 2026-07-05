// ── Files Explorer ── Tauri backend entry point ──

mod commands;
mod core;
pub(crate) mod platform;
mod utils;

mod clipboard;
mod compress;
mod search;
mod tray;
mod undo;

pub mod tauri_setup;

use core::error::FsError;
use core::state::{AppState, AppStateInner};
use core::types::{ClipboardInfo, DiskInfo, FileAction, FileEntry, SpecialDirs};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use tauri::{command, AppHandle, Emitter, Manager, RunEvent, State};

use tauri_setup::SetupState;

#[command]
async fn set_complete(
    app: AppHandle,
    state: State<'_, Mutex<SetupState>>,
    task: String,
) -> Result<(), String> {
    let mut state_lock = state.lock().map_err(|e| e.to_string())?;
    match task.as_str() {
        "frontend" => state_lock.frontend_task = true,
        "backend" => state_lock.backend_task = true,
        _ => return Err("invalid task".into()),
    }
    if state_lock.backend_task && state_lock.frontend_task {
        if let Some(splash) = app.get_webview_window("splashscreen") {
            let _ = splash.close();
        }
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
            let _ = main.set_focus();
        }
    }
    Ok(())
}

// ── Files ──
#[command]
fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    use std::path::Path;
    crate::core::fs_helper::list_directory(Path::new(&path), false).map_err(|e| e.to_string())
}
#[command]
fn list_directory_streamed(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<(), String> {
    crate::commands::file_cmd::list_directory_streamed(app, state, path)
}
#[command]
fn get_file_info(path: String) -> Result<FileEntry, String> {
    crate::core::fs_helper::file_entry_from_path(std::path::Path::new(&path))
        .map_err(|e| e.to_string())
}
#[command]
fn path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

// ── Drives ──
#[command]
fn get_drives() -> Result<Vec<DiskInfo>, String> {
    Ok(crate::platform::system_provider().get_drives())
}
#[command]
fn get_special_dirs() -> Result<SpecialDirs, String> {
    let p = crate::platform::path_provider();
    Ok(SpecialDirs {
        home: p.home_dir().to_string_lossy().to_string(),
        desktop: p.desktop_dir().to_string_lossy().to_string(),
        documents: p.documents_dir().to_string_lossy().to_string(),
        downloads: p.downloads_dir().to_string_lossy().to_string(),
        pictures: p.pictures_dir().to_string_lossy().to_string(),
        music: p.music_dir().to_string_lossy().to_string(),
        videos: p.videos_dir().to_string_lossy().to_string(),
    })
}

// ── Operations ──
#[command]
fn get_parent_directory(path: String) -> Result<String, FsError> {
    let p = std::path::Path::new(&path);
    match p.parent() {
        Some(parent) => Ok(parent.to_string_lossy().to_string()),
        None => Err(FsError::InvalidPath("No parent directory".into())),
    }
}
#[command]
fn create_directory(state: State<AppState>, path: String) -> Result<(), FsError> {
    crate::commands::file_cmd::create_directory(state, path)
}
#[command]
fn create_file(state: State<AppState>, path: String) -> Result<(), FsError> {
    crate::commands::file_cmd::create_file(state, path)
}
#[command]
fn delete_item(path: String, permanently: bool) -> Result<(), FsError> {
    crate::commands::file_cmd::delete_item(path, permanently)
}
#[command]
fn rename_item(state: State<AppState>, old_path: String, new_path: String) -> Result<(), FsError> {
    crate::commands::file_cmd::rename_item(state, old_path, new_path)
}
#[command]
fn move_files(paths: Vec<String>, dest_dir: String, copy: bool) -> Result<(), FsError> {
    crate::commands::file_cmd::move_files(paths, dest_dir, copy)
}

// ── Clipboard ──
#[command]
fn copy_clipboard(state: State<AppState>, paths: Vec<String>) -> Result<(), FsError> {
    clipboard::copy_clipboard(state, paths)
}
#[command]
fn cut_clipboard(state: State<AppState>, paths: Vec<String>) -> Result<(), FsError> {
    clipboard::cut_clipboard(state, paths)
}
#[command]
fn paste_clipboard(state: State<AppState>, dest_dir: String) -> Result<(), FsError> {
    clipboard::paste_clipboard(state, dest_dir)
}
#[command]
fn get_clipboard_info(state: State<AppState>) -> Result<ClipboardInfo, FsError> {
    clipboard::get_clipboard_info(state)
}

// ── Search ──
#[command]
fn search_files(
    app: AppHandle,
    state: State<AppState>,
    directory: String,
    query: String,
) -> Result<(), String> {
    search::search_files(app, state, directory, query)
}
#[command]
fn cancel_search(state: State<AppState>) -> Result<(), String> {
    search::cancel_search(state)
}

// ── System (delegated to platform trait) ──
#[command]
fn open_file(path: String) -> Result<(), String> {
    crate::platform::system_provider().open_file(std::path::Path::new(&path))
}
#[command]
fn open_in_terminal(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let dir = if p.is_dir() {
        p
    } else {
        p.parent().unwrap_or(p)
    };
    crate::platform::system_provider().open_terminal(dir)
}
#[command]
fn show_in_explorer(path: String) -> Result<(), String> {
    crate::platform::system_provider().show_in_file_manager(std::path::Path::new(&path))
}
#[command]
fn show_file_properties(path: String) -> Result<(), String> {
    crate::platform::system_provider().show_properties(std::path::Path::new(&path))
}
#[command]
fn print_file(path: String) -> Result<(), String> {
    crate::platform::system_provider().print_file(std::path::Path::new(&path))
}
#[command]
fn get_file_icon(path: String) -> Result<String, String> {
    use base64::Engine;
    let png = crate::platform::system_provider().get_file_icon(std::path::Path::new(&path))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&png))
}
#[command]
fn start_native_drag_cmd(paths: Vec<String>) -> Result<String, String> {
    crate::platform::system_provider().start_native_drag(&paths)
}

// ── Auto-start ──
#[command]
fn set_auto_start(enabled: bool) -> Result<(), String> {
    crate::platform::system_provider().set_auto_start(enabled)
}
#[command]
fn is_auto_start_enabled() -> bool {
    crate::platform::system_provider().is_auto_start_enabled()
}

// ── Compress ──
#[command]
fn compress_files(app: AppHandle, paths: Vec<String>, dest: String) -> Result<(), FsError> {
    let cancel = Arc::new(AtomicBool::new(false));
    compress::compress_zip(app, paths, dest, cancel)
}
#[command]
fn extract_archive_cmd(app: AppHandle, archive: String, dest_dir: String) -> Result<(), FsError> {
    let cancel = Arc::new(AtomicBool::new(false));
    compress::extract_archive(app, archive, dest_dir, cancel)
}

// ── Archive browsing (from compress) ──
#[command]
fn list_archive_contents(path: String) -> Result<Vec<compress::ArchiveEntry>, String> {
    compress::list_archive_contents(path)
}
#[command]
fn extract_archive_entry(
    archive_path: String,
    entry_path: String,
) -> Result<compress::ExtractResult, String> {
    compress::extract_archive_entry(archive_path, entry_path)
}

// ── File preview ──
#[command]
fn get_file_base64(path: String) -> Result<serde_json::Value, String> {
    use base64::Engine;
    let bytes = std::fs::read(&path).map_err(|e| format!("Read failed: {}", e))?;
    const MAX: usize = 2 * 1024 * 1024;
    if bytes.len() > MAX {
        return Err("File too large for preview".to_string());
    }
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => "image/png",
    };
    Ok(
        serde_json::json!({"mime":mime,"data":base64::engine::general_purpose::STANDARD.encode(&bytes)}),
    )
}

#[command]
fn read_file_bytes(path: String) -> Result<String, String> {
    use base64::Engine;
    let m = std::fs::metadata(&path).map_err(|e| format!("Stat: {}", e))?;
    const MAX: u64 = 20 * 1024 * 1024;
    if m.len() > MAX {
        return Err("File too large".to_string());
    }
    Ok(base64::engine::general_purpose::STANDARD
        .encode(&std::fs::read(&path).map_err(|e| format!("Read: {}", e))?))
}

#[command]
fn get_file_preview(path: String) -> Result<serde_json::Value, String> {
    use crate::utils::encoding::{
        is_known_binary_ext, is_known_text_ext, is_probably_text, truncate_to_chars,
    };
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if is_known_binary_ext(&ext) {
        return Err("Binary file".into());
    }
    const MAX: usize = 512 * 1024;
    const CHARS: usize = 10000;
    let m = std::fs::metadata(&path).map_err(|e| format!("Stat: {}", e))?;
    if m.len() > MAX as u64 {
        return Err("File too large".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("Read: {}", e))?;
    if !is_known_text_ext(&ext) && !is_probably_text(&bytes) {
        return Err("Binary file".into());
    }
    let content = String::from_utf8_lossy(&bytes);
    let preview = truncate_to_chars(&content, CHARS).to_string();
    let is_md = ext == "md"
        || ext == "mdx"
        || (preview.starts_with('#') && preview.contains('[') && preview.contains("]("));
    Ok(serde_json::json!({"type":if is_md{"markdown"}else{"text"},"content":preview,"ext":ext}))
}

#[command]
fn copy_file_as(src: String, dest: String) -> Result<(), String> {
    std::fs::copy(&src, &dest).map_err(|e| format!("Copy: {}", e))?;
    Ok(())
}

#[command]
fn save_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Save: {}", e))
}

// ── Undo ──
#[command]
fn undo_last_action(state: State<AppState>) -> Result<String, FsError> {
    undo::undo_last_action(state)
}
#[command]
fn get_undo_info(state: State<AppState>) -> Result<Option<FileAction>, FsError> {
    undo::get_undo_info(state)
}

#[command]
fn set_tray_visible(visible: bool, app: AppHandle) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_visible(visible);
    }
}

#[command]
fn set_quit_on_close(state: State<AppState>, enabled: bool) {
    state.quit_on_close.store(enabled, Ordering::SeqCst);
}

#[command]
fn clear_window_state(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed: {}", e))?;
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed: {}", e))?;
    }
    Ok(())
}

// ── Entry ──
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri_setup::build_app().run(|app, event| {
        tauri_setup::run_callback(app, event);
    });
}
