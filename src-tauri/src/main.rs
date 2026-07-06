// main.rs — Tauri entry-point: assembly only, zero platform logic.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    files_explorer_lib::tauri_setup::init_logging();
    tracing::info!("Application starting");

    let app = files_explorer_lib::tauri_setup::build_app();
    app.run(|app_handle, event| {
        files_explorer_lib::tauri_setup::run_callback(app_handle, event);
    });
}
