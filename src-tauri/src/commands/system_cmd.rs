// commands/system_cmd.rs
// System-related Tauri commands — delegate to platform trait.

use crate::core::types::{DiskInfo, SpecialDirs};
use crate::platform;

/// Internal helper used by tray.rs to get special dirs without the command wrapper.
pub fn get_special_dirs_impl() -> SpecialDirs {
    let p = platform::path_provider();
    SpecialDirs {
        home: p.home_dir().to_string_lossy().to_string(),
        desktop: p.desktop_dir().to_string_lossy().to_string(),
        documents: p.documents_dir().to_string_lossy().to_string(),
        downloads: p.downloads_dir().to_string_lossy().to_string(),
        pictures: p.pictures_dir().to_string_lossy().to_string(),
        music: p.music_dir().to_string_lossy().to_string(),
        videos: p.videos_dir().to_string_lossy().to_string(),
    }
}

pub fn open_file(path: String) -> Result<(), String> {
    platform::system_provider().open_file(std::path::Path::new(&path))
}

pub fn open_in_terminal(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let dir = if p.is_dir() {
        p
    } else {
        p.parent().unwrap_or(p)
    };
    platform::system_provider().open_terminal(dir)
}

pub fn show_in_explorer(path: String) -> Result<(), String> {
    platform::system_provider().show_in_file_manager(std::path::Path::new(&path))
}

pub fn show_file_properties(path: String) -> Result<(), String> {
    platform::system_provider().show_properties(std::path::Path::new(&path))
}

pub fn print_file(path: String) -> Result<(), String> {
    platform::system_provider().print_file(std::path::Path::new(&path))
}

pub fn get_file_icon(path: String) -> Result<String, String> {
    let png = platform::system_provider().get_file_icon(std::path::Path::new(&path))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png,
    ))
}

pub fn get_drives() -> Result<Vec<DiskInfo>, String> {
    Ok(platform::system_provider().get_drives())
}

pub fn get_special_dirs() -> Result<SpecialDirs, String> {
    Ok(get_special_dirs_impl())
}

pub fn set_auto_start(enabled: bool) -> Result<(), String> {
    platform::system_provider().set_auto_start(enabled)
}

pub fn is_auto_start_enabled() -> bool {
    platform::system_provider().is_auto_start_enabled()
}

pub fn start_native_drag_cmd(paths: Vec<String>) -> Result<String, String> {
    platform::system_provider().start_native_drag(&paths)
}

pub fn clear_window_state(_app: tauri::AppHandle) -> Result<(), String> {
    let dir = crate::platform::path_provider().app_data_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed: {}", e))?;
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed: {}", e))?;
    }
    Ok(())
}
