// platform/path.rs
// Unified cross-platform directory abstraction.
// Each platform provides a singleton implementing this trait.

use std::path::PathBuf;

pub trait PlatformPath: Send + Sync {
    /// Persistent app data (config, databases, state).
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | Windows  | `%APPDATA%/files-explorer` |
    /// | macOS    | `~/Library/Application Support/files-explorer` |
    /// | Linux    | `~/.local/share/files-explorer` |
    fn app_data_dir(&self) -> PathBuf;

    /// Cache directory (may be purged by the OS at any time).
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | Windows  | `%LOCALAPPDATA%/files-explorer/cache` |
    /// | macOS    | `~/Library/Caches/files-explorer` |
    /// | Linux    | `~/.cache/files-explorer` |
    fn app_cache_dir(&self) -> PathBuf;

    /// Config directory.
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | Windows  | `%APPDATA%/files-explorer/config` |
    /// | macOS    | `~/Library/Application Support/files-explorer/config` |
    /// | Linux    | `~/.config/files-explorer` |
    fn app_config_dir(&self) -> PathBuf;

    /// Log directory.
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | Windows  | `%APPDATA%/files-explorer/logs` |
    /// | macOS    | `~/Library/Logs/files-explorer` |
    /// | Linux    | `~/.local/share/files-explorer/logs` |
    fn app_log_dir(&self) -> PathBuf;

    /// Bundled resource directory (next to the executable, or inside .app bundle).
    fn app_resource_dir(&self) -> PathBuf;

    // ── User well-known folders ──

    fn home_dir(&self) -> PathBuf;
    fn desktop_dir(&self) -> PathBuf;
    fn documents_dir(&self) -> PathBuf;
    fn downloads_dir(&self) -> PathBuf;
    fn pictures_dir(&self) -> PathBuf;
    fn music_dir(&self) -> PathBuf;
    fn videos_dir(&self) -> PathBuf;

    /// Temporary directory used for preview extractions and scratch files.
    fn temp_dir(&self) -> PathBuf;
}
