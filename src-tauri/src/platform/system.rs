// platform/system.rs
// Unified system capability abstraction.
// Covers: file launching, terminal, file manager, properties, print,
//   icons, auto-start, clipboard, native drag, drives, tray, notifications.

use crate::core::types::DiskInfo;
use std::path::{Path, PathBuf};

pub trait PlatformSystem: Send + Sync {
    // ── File / app launching ──

    /// Open a file with the default OS handler.
    fn open_file(&self, path: &Path) -> Result<(), String>;

    /// Open a terminal window at the given directory.
    fn open_terminal(&self, dir: &Path) -> Result<(), String>;

    /// Reveal a file or folder in the system file manager.
    fn show_in_file_manager(&self, path: &Path) -> Result<(), String>;

    /// Open the OS file-properties dialog for a path.
    fn show_properties(&self, path: &Path) -> Result<(), String>;

    /// Send a file to the system print queue.
    fn print_file(&self, path: &Path) -> Result<(), String>;

    // ── File icon ──

    /// Return the native file icon as raw PNG bytes.
    fn get_file_icon(&self, path: &Path) -> Result<Vec<u8>, String>;

    // ── Auto-start ──

    fn set_auto_start(&self, enabled: bool) -> Result<(), String>;
    fn is_auto_start_enabled(&self) -> bool;

    // ── Clipboard (file paths) ──

    /// Copy file paths to the system clipboard (best-effort).
    fn write_clipboard(&self, paths: &[PathBuf]);
    /// Read file paths from the system clipboard, if available.
    fn read_clipboard(&self) -> Option<Vec<PathBuf>>;

    // ── Native drag-out (Windows COM, others unsupported) ──

    fn start_native_drag(&self, paths: &[String]) -> Result<String, String>;

    // ── Drive / volume enumeration ──

    fn get_drives(&self) -> Vec<DiskInfo>;

    // ── Terminal / shell ──

    /// Return the path to the default system shell.
    fn default_shell(&self) -> String;

    // ── System tray support ──

    /// Whether the system tray is reliably available.
    /// Linux Wayland compositors may not support it.
    fn is_tray_supported(&self) -> bool;

    // ── System notifications ──

    fn send_notification(&self, title: &str, body: &str) -> Result<(), String>;

    // ── Global shortcuts (reserved) ──

    /// Register a global keyboard shortcut.
    /// macOS requires Accessibility permission.
    fn register_global_shortcut(
        &self,
        _modifiers: u32,
        _key: u32,
        _callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        Err("Global shortcuts not implemented on this platform".into())
    }
}
