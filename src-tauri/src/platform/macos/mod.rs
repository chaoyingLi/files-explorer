// platform/macos/mod.rs
// macOS platform implementations.

mod fs_impl;
mod path_impl;
mod system_impl;

pub use fs_impl::FsExtImpl;
pub use path_impl::PathImpl;
pub use system_impl::SystemImpl;

use crate::platform::PlatformWindow;
use tauri::AppHandle;

pub struct WindowImpl;

impl PlatformWindow for WindowImpl {
    fn install_lifecycle_hooks(&self, _app: &AppHandle) {
        // macOS: no-op, Dock-click Reopen is handled in handle_run_event.
    }

    fn handle_run_event(&self, app: &AppHandle, event: &tauri::RunEvent) {
        if let tauri::RunEvent::Reopen { .. } = event {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }
}

impl WindowImpl {
    pub fn instance() -> &'static WindowImpl {
        &WindowImpl
    }
}
