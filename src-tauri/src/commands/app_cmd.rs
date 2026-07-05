// commands/app_cmd.rs

use crate::core::state::AppState;
use crate::tauri_setup::SetupState;
use std::sync::{atomic::Ordering, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

pub async fn set_complete(
    app: AppHandle,
    state: State<'_, Mutex<SetupState>>,
    task: String,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    match task.as_str() {
        "frontend" => s.frontend_task = true,
        "backend" => s.backend_task = true,
        _ => return Err("invalid task".into()),
    }
    if s.backend_task && s.frontend_task {
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

pub fn set_tray_visible(visible: bool, app: AppHandle) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_visible(visible);
    }
}

pub fn set_quit_on_close(state: State<AppState>, enabled: bool) {
    state.quit_on_close.store(enabled, Ordering::SeqCst);
}
