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
        // macOS lifecycle hooks (Dock click → Reopen) are handled in
        // tauri_setup.rs::run_callback via the RunEvent dispatcher,
        // keeping the platform layer free of Tauri runtime details.
    }
}

impl WindowImpl {
    pub fn instance() -> &'static WindowImpl {
        &WindowImpl
    }
}
