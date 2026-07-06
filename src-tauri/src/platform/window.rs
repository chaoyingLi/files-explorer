// platform/window.rs
// Platform-specific window lifecycle behaviour.

use tauri::AppHandle;

pub trait PlatformWindow: Send + Sync {
    /// Register platform-specific event handlers on the Tauri app.
    /// Called once during `tauri_setup`.
    ///
    /// | Platform | Hook |
    /// |----------|------|
    /// | macOS    | `RunEvent::Reopen` → show window on Dock click |
    /// | Windows  | No-op (close→tray handled in `tauri_setup`) |
    /// | Linux    | No-op |
    fn install_lifecycle_hooks(&self, app: &AppHandle);

    /// Handle a Tauri `RunEvent` for platform-specific behaviour.
    ///
    /// Called from `tauri_setup::run_callback` for every event.
    /// The default implementation is a no-op.
    ///
    /// | Platform | Event |
    /// |----------|-------|
    /// | macOS    | `Reopen` → show window on Dock click |
    /// | Windows  | No-op |
    /// | Linux    | No-op |
    fn handle_run_event(&self, app_handle: &AppHandle, _event: &tauri::RunEvent) {
        let _ = app_handle;
    }
}
