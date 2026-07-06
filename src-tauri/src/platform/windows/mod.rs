// platform/windows/mod.rs
// Windows platform implementations.

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
        // Windows: no special lifecycle hooks needed.
        // The close→minimize-to-tray behaviour is handled in tauri_setup.rs.
    }
}

impl WindowImpl {
    pub fn instance() -> &'static WindowImpl {
        &WindowImpl
    }
}
