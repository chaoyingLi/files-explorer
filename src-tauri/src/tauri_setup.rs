// tauri_setup.rs
// Centralised Tauri Builder initialisation.
// All platform-specific lifecycle logic is delegated to `platform`.

use crate::core::logger;
use crate::core::state::{AppState, AppStateInner};
use crate::platform;
use crate::tray;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use tauri::{Emitter, Manager, RunEvent};

pub struct SetupState {
    pub frontend_task: bool,
    pub backend_task: bool,
}

/// Initialise logging. Delegates entirely to core::logger.
pub fn init_logging() {
    logger::init();
    tracing::info!("=== Files Explorer started ===");
}

/// Build the Tauri application.
pub fn build_app() -> tauri::App {
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
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            crate::list_directory,
            crate::list_directory_streamed,
            crate::get_drives,
            crate::get_parent_directory,
            crate::create_directory,
            crate::create_file,
            crate::delete_item,
            crate::rename_item,
            crate::copy_clipboard,
            crate::cut_clipboard,
            crate::paste_clipboard,
            crate::get_file_info,
            crate::open_file,
            crate::get_file_base64,
            crate::show_in_explorer,
            crate::start_native_drag_cmd,
            crate::show_file_properties,
            crate::get_file_preview,
            crate::get_file_icon,
            crate::read_file_bytes,
            crate::list_archive_contents,
            crate::extract_archive_entry,
            crate::print_file,
            crate::copy_file_as,
            crate::save_text_file,
            crate::open_in_terminal,
            crate::search_files,
            crate::path_exists,
            crate::get_special_dirs,
            crate::get_clipboard_info,
            crate::undo_last_action,
            crate::get_undo_info,
            crate::cancel_search,
            crate::move_files,
            crate::compress_files,
            crate::extract_archive_cmd,
            crate::set_auto_start,
            crate::is_auto_start_enabled,
            crate::set_tray_visible,
            crate::set_quit_on_close,
            crate::set_complete,
            crate::clear_window_state,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
}

/// Setup callback: tray, window lifecycle, platform hooks.
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    tray::create_tray(app.handle())?;

    setup_window_close(app.handle());

    // Install platform-specific lifecycle hooks (e.g. macOS Dock reopen).
    platform::window_provider().install_lifecycle_hooks(app.handle());

    // Signal backend ready
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::set_complete(
            handle.clone(),
            handle.state::<Mutex<SetupState>>(),
            "backend".to_string(),
        )
        .await;
    });

    Ok(())
}

/// Intercept main window close → hide to tray (unless quit_on_close).
fn setup_window_close(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let w = win.clone();
        let handle = app.clone();
        let quit_flag = app.state::<AppState>().quit_on_close.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if quit_flag.load(Ordering::SeqCst) {
                    return;
                }
                api.prevent_close();
                let _ = w.hide();
                let _ = tray::rebuild_tray(&handle, false);
                let _ = w.emit("tray-hide", ());
            }
        });
    }
}

/// Run-event callback for platform-specific behaviour (e.g. macOS Dock Reopen).
pub fn run_callback(app_handle: &tauri::AppHandle, event: RunEvent) {
    // Platform-specific lifecycle hooks via the platform trait.
    // The Reopen event is the only one that differs across platforms.
    match event {
        RunEvent::Reopen { .. } => {
            if let Some(w) = app_handle.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        _ => {}
    }
}
