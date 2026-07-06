// platform/linux/mod.rs
// Linux platform implementations.

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
        // Linux: no special lifecycle hooks.
    }
}

impl WindowImpl {
    pub fn instance() -> &'static WindowImpl {
        &WindowImpl
    }
}
