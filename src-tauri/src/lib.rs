// ── Files Explorer ── Tauri backend entry point ──

mod autostart;
mod clipboard;
mod compress;
mod drives;
mod error;
mod files;
mod native_drag;
mod operations;
mod search;
mod state;
mod system;
mod tray;
mod types;
mod undo;

use state::{AppState, AppStateInner};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use tauri::{command, AppHandle, Emitter, Manager, State};

use crate::error::FsError;
use crate::types::{ClipboardInfo, DiskInfo, FileAction, FileEntry, SpecialDirs};

// ── Splashscreen state ──
struct SetupState {
    frontend_task: bool,
    backend_task: bool,
}

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
    files::list_directory(path)
}
#[command]
fn list_directory_streamed(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<(), String> {
    files::list_directory_streamed(app, path, state.navigate_gen.clone())
}
#[command]
fn get_file_info(path: String) -> Result<FileEntry, String> {
    files::get_file_info(path)
}
#[command]
fn path_exists(path: String) -> bool {
    files::path_exists(path)
}

// ── Drives ──
#[command]
fn get_drives() -> Result<Vec<DiskInfo>, String> {
    drives::get_drives()
}
#[command]
fn get_special_dirs() -> Result<SpecialDirs, String> {
    drives::get_special_dirs()
}

// ── Operations ──
#[command]
fn get_parent_directory(path: String) -> Result<String, FsError> {
    operations::get_parent_directory(path)
}
#[command]
fn create_directory(state: State<AppState>, path: String) -> Result<(), FsError> {
    operations::create_directory(state, path)
}
#[command]
fn create_file(state: State<AppState>, path: String) -> Result<(), FsError> {
    operations::create_file(state, path)
}
#[command]
fn delete_item(path: String, permanently: bool) -> Result<(), FsError> {
    operations::delete_item(path, permanently)
}
#[command]
fn rename_item(state: State<AppState>, old_path: String, new_path: String) -> Result<(), FsError> {
    operations::rename_item(state, old_path, new_path)
}
#[command]
fn move_files(paths: Vec<String>, dest_dir: String, copy: bool) -> Result<(), FsError> {
    operations::move_files(paths, dest_dir, copy)
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

// ── System ──
#[command]
fn open_file(path: String) -> Result<(), String> {
    system::open_file(path)
}
#[command]
fn open_in_terminal(path: String) -> Result<(), String> {
    system::open_in_terminal(path)
}
#[command]
fn show_in_explorer(path: String) -> Result<(), String> {
    system::show_in_explorer(path)
}
#[command]
fn show_file_properties(path: String) -> Result<(), String> {
    system::show_file_properties(path)
}
#[command]
fn get_file_preview(path: String) -> Result<serde_json::Value, String> {
    system::get_file_preview(path)
}
#[command]
fn read_file_bytes(path: String) -> Result<String, String> {
    system::read_file_bytes(path)
}
#[command]
fn list_archive_contents(path: String) -> Result<Vec<system::ArchiveEntry>, String> {
    system::list_archive_contents(path)
}
#[command]
fn extract_archive_entry(
    archive_path: String,
    entry_path: String,
) -> Result<system::ExtractResult, String> {
    system::extract_archive_entry(archive_path, entry_path)
}
#[command]
fn print_file(path: String) -> Result<(), String> {
    system::print_file(path)
}
#[command]
fn copy_file_as(src: String, dest: String) -> Result<(), String> {
    system::copy_file_as(src, dest)
}
#[command]
fn save_text_file(path: String, content: String) -> Result<(), String> {
    system::save_text_file(path, content)
}
#[command]
fn get_file_icon(path: String) -> Result<String, String> {
    system::get_file_icon(path)
}
#[command]
fn get_file_base64(path: String) -> Result<serde_json::Value, String> {
    system::get_file_base64(path)
}
#[command]
fn start_native_drag_cmd(paths: Vec<String>) -> Result<String, String> {
    system::start_native_drag_cmd(paths)
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

// ── Undo ──
#[command]
fn undo_last_action(state: State<AppState>) -> Result<String, FsError> {
    undo::undo_last_action(state)
}
#[command]
fn get_undo_info(state: State<AppState>) -> Result<Option<FileAction>, FsError> {
    undo::get_undo_info(state)
}

// ── Auto-start ──
#[command]
fn set_auto_start(enabled: bool) -> Result<(), String> {
    autostart::set_auto_start(enabled)
}
#[command]
fn is_auto_start_enabled() -> bool {
    autostart::is_auto_start_enabled()
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

// ── Entry ──
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(AppState {
            inner: Mutex::new(AppStateInner {
                clipboard: Vec::new(),
                clipboard_action: String::new(),
                undo_history: Vec::new(),
            }),
            search_cancel: Arc::new(AtomicBool::new(false)),
            navigate_gen: Arc::new(AtomicU64::new(0)),
            quit_on_close: Arc::new(AtomicBool::new(false)),
        })
        .manage(Mutex::new(SetupState {
            frontend_task: false,
            backend_task: false,
        }))
        .setup(|app| {
            tray::create_tray(app.handle())?;
            // Intercept window close: check quit_on_close flag
            if let Some(win) = app.get_webview_window("main") {
                let w = win.clone();
                let quit_flag = app.state::<AppState>().quit_on_close.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        if quit_flag.load(Ordering::SeqCst) {
                            // Let window close normally
                            return;
                        }
                        api.prevent_close();
                        let _ = w.hide();
                        let _ = w.emit("tray-hide", ());
                    }
                });
            }
            // Signal backend ready
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = set_complete(
                    handle.clone(),
                    handle.state::<Mutex<SetupState>>(),
                    "backend".to_string(),
                )
                .await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_directory,
            list_directory_streamed,
            get_drives,
            get_parent_directory,
            create_directory,
            create_file,
            delete_item,
            rename_item,
            copy_clipboard,
            cut_clipboard,
            paste_clipboard,
            get_file_info,
            open_file,
            get_file_base64,
            show_in_explorer,
            start_native_drag_cmd,
            show_file_properties,
            get_file_preview,
            get_file_icon,
            read_file_bytes,
            list_archive_contents,
            extract_archive_entry,
            print_file,
            copy_file_as,
            save_text_file,
            open_in_terminal,
            search_files,
            path_exists,
            get_special_dirs,
            get_clipboard_info,
            undo_last_action,
            get_undo_info,
            cancel_search,
            move_files,
            compress_files,
            extract_archive_cmd,
            set_auto_start,
            is_auto_start_enabled,
            set_tray_visible,
            set_quit_on_close,
            set_complete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
