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
}
