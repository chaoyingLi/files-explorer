// ── Files Explorer ── Tauri backend entry point ──

mod commands;
mod core;
pub(crate) mod platform;
mod utils;

mod clipboard;
mod compress;
mod search;
mod terminal;
mod tray;
mod undo;

pub mod tauri_setup;

use core::error::FsError;
use core::fs_helper;
use core::state::AppState;
use core::types::{ClipboardInfo, DiskInfo, FileAction, FileEntry, SpecialDirs};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{command, AppHandle, Manager, State};

use tauri_setup::SetupState;

#[command]
async fn set_complete(
    app: AppHandle,
    state: State<'_, Mutex<SetupState>>,
    task: String,
) -> Result<(), FsError> {
    let mut state_lock = state.lock().map_err(|e| FsError::Other(e.to_string()))?;
    match task.as_str() {
        "frontend" => state_lock.frontend_task = true,
        "backend" => state_lock.backend_task = true,
        _ => return Err(FsError::Other("invalid task".into())),
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
fn list_directory(path: String) -> Result<Vec<FileEntry>, FsError> {
    use std::path::Path;
    crate::core::fs_helper::list_directory(Path::new(&path), false)
}
#[command]
fn list_directory_streamed(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<(), FsError> {
    crate::commands::file_cmd::list_directory_streamed(app, state, path)
        .map_err(|e| FsError::Other(e))
}
#[command]
fn get_file_info(path: String) -> Result<FileEntry, FsError> {
    crate::core::fs_helper::file_entry_from_path(std::path::Path::new(&path))
}
#[command]
fn path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

// ── Drives ──
#[command]
fn get_drives() -> Result<Vec<DiskInfo>, FsError> {
    Ok(crate::platform::system_provider().get_drives())
}
#[command]
fn get_special_dirs() -> Result<SpecialDirs, FsError> {
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
) -> Result<(), FsError> {
    search::search_files(app, state, directory, query)
}
#[command]
fn cancel_search(state: State<AppState>) -> Result<(), FsError> {
    search::cancel_search(state)
}

// ── System (delegated to platform trait) ──
#[command]
fn open_file(path: String) -> Result<(), FsError> {
    crate::platform::system_provider()
        .open_file(std::path::Path::new(&path))
        .map_err(FsError::Other)
}
#[command]
fn open_in_terminal(path: String) -> Result<(), FsError> {
    let p = std::path::Path::new(&path);
    let dir = if p.is_dir() {
        p
    } else {
        p.parent().unwrap_or(p)
    };
    crate::platform::system_provider()
        .open_terminal(dir)
        .map_err(FsError::Other)
}
#[command]
fn show_in_explorer(path: String) -> Result<(), FsError> {
    crate::platform::system_provider()
        .show_in_file_manager(std::path::Path::new(&path))
        .map_err(FsError::Other)
}
#[command]
fn show_file_properties(path: String) -> Result<(), FsError> {
    crate::platform::system_provider()
        .show_properties(std::path::Path::new(&path))
        .map_err(FsError::Other)
}
#[command]
fn print_file(path: String) -> Result<(), FsError> {
    crate::platform::system_provider()
        .print_file(std::path::Path::new(&path))
        .map_err(FsError::Other)
}
#[command]
fn get_file_icon(path: String) -> Result<String, FsError> {
    use base64::Engine;
    let png = crate::platform::system_provider()
        .get_file_icon(std::path::Path::new(&path))
        .map_err(FsError::Other)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&png))
}
#[command]
fn start_native_drag_cmd(paths: Vec<String>) -> Result<String, FsError> {
    crate::platform::system_provider()
        .start_native_drag(&paths)
        .map_err(FsError::Other)
}

// ── Auto-start ──
#[command]
fn set_auto_start(enabled: bool) -> Result<(), FsError> {
    crate::platform::system_provider()
        .set_auto_start(enabled)
        .map_err(FsError::Other)
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
fn list_archive_contents(path: String) -> Result<Vec<compress::ArchiveEntry>, FsError> {
    compress::list_archive_contents(path).map_err(FsError::Other)
}
#[command]
fn extract_archive_entry(
    archive_path: String,
    entry_path: String,
) -> Result<compress::ExtractResult, FsError> {
    compress::extract_archive_entry(archive_path, entry_path).map_err(FsError::Other)
}

// ── Terminal ──
#[command]
fn terminal_spawn(app: AppHandle, id: u32, cwd: String, term_type: String) -> Result<(), FsError> {
    terminal::terminal_state()
        .spawn(id, app, std::path::Path::new(&cwd), &term_type)
        .map_err(|e| e)
}

#[command]
fn terminal_write(id: u32, data: String) -> Result<(), FsError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| FsError::Other(format!("Decode: {}", e)))?;
    terminal::terminal_state().write(id, &bytes).map_err(|e| e)
}

#[command]
fn terminal_resize(id: u32, rows: u16, cols: u16) -> Result<(), FsError> {
    terminal::terminal_state()
        .resize(id, rows, cols)
        .map_err(|e| e)
}

#[command]
fn terminal_kill(id: u32) {
    terminal::terminal_state().kill(id);
}

#[command]
fn terminal_kill_all() {
    terminal::terminal_state().kill_all();
}
#[command]
fn get_default_shell() -> String {
    platform::system_provider().default_shell()
}

// ── File preview ──
#[command]
fn get_file_base64(path: String) -> Result<serde_json::Value, FsError> {
    use base64::Engine;
    let p = std::path::Path::new(&path);
    let bytes = fs_helper::read_file_limited(p, 2 * 1024 * 1024)
        .map_err(|e| FsError::IoError(format!("Read failed: {}", e)))?;
    let ext = p
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
fn read_file_bytes(path: String) -> Result<String, FsError> {
    use base64::Engine;
    let p = std::path::Path::new(&path);
    let m = fs_helper::metadata(p).map_err(|e| FsError::IoError(format!("Stat: {}", e)))?;
    const MAX: u64 = 20 * 1024 * 1024;
    if m.len() > MAX {
        return Err(FsError::Other("File too large".to_string()));
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(
        &fs_helper::read_file_limited(p, MAX as usize)
            .map_err(|e| FsError::IoError(format!("Read: {}", e)))?,
    ))
}

#[command]
fn get_file_preview(path: String) -> Result<serde_json::Value, FsError> {
    use crate::utils::encoding::{
        decode_to_utf8, is_known_binary_ext, is_known_text_ext, is_probably_text, truncate_to_chars,
    };
    let p = std::path::Path::new(&path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if is_known_binary_ext(&ext) {
        return Err(FsError::Other("Binary file".into()));
    }
    const MAX: usize = 512 * 1024;
    const CHARS: usize = 10000;
    let m = fs_helper::metadata(p).map_err(|e| FsError::IoError(format!("Stat: {}", e)))?;
    if m.len() > MAX as u64 {
        return Err(FsError::Other("File too large".into()));
    }
    let bytes = fs_helper::read_file_limited(p, MAX)
        .map_err(|e| FsError::IoError(format!("Read: {}", e)))?;
    if !is_known_text_ext(&ext) && !is_probably_text(&bytes) {
        return Err(FsError::Other("Binary file".into()));
    }
    // ── Encoding-aware decode (BOM detection + chardetng + encoding_rs) ──
    let content = decode_to_utf8(&bytes);
    let preview = truncate_to_chars(&content, CHARS).to_string();
    let is_md = ext == "md"
        || ext == "mdx"
        || (preview.starts_with('#') && preview.contains('[') && preview.contains("]("));
    Ok(serde_json::json!({"type":if is_md{"markdown"}else{"text"},"content":preview,"ext":ext}))
}

#[command]
fn copy_file_as(src: String, dest: String) -> Result<(), FsError> {
    fs_helper::copy_file(std::path::Path::new(&src), std::path::Path::new(&dest))
        .map_err(|e| FsError::IoError(format!("Copy: {}", e)))?;
    Ok(())
}

#[command]
fn save_text_file(path: String, content: String) -> Result<(), FsError> {
    fs_helper::write_file(std::path::Path::new(&path), content.as_bytes())
        .map_err(|e| FsError::IoError(format!("Save: {}", e)))
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
fn clear_window_state() -> Result<(), FsError> {
    let dir = crate::platform::path_provider().app_data_dir();
    if dir.exists() {
        fs_helper::remove_dir_all(&dir)?;
        fs_helper::create_dir_all(&dir)?;
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
