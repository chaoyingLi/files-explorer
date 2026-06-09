use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, State};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub file_system: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopyProgress {
    pub current: u64,
    pub total: u64,
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ActionKind {
    Delete,
    Rename { old_path: String, new_path: String },
    Create { path: String, is_dir: bool },
    Copy { src: String, dest: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileAction {
    pub kind: ActionKind,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardInfo {
    pub paths: Vec<String>,
    pub action: String, // "copy" or "cut"
}

struct AppState {
    clipboard: std::sync::Mutex<Vec<PathBuf>>,
    clipboard_action: std::sync::Mutex<String>, // "copy" or "cut"
    undo_history: std::sync::Mutex<Vec<FileAction>>,
}

#[command]
fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();

    match fs::read_dir(&path) {
        Ok(read_dir) => {
            for entry in read_dir.flatten() {
                let file_path = entry.path();
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => match fs::metadata(&file_path) {
                        Ok(m) => m,
                        Err(_) => continue,
                    },
                };

                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = metadata.is_dir();
                let size = if is_dir { 0 } else { metadata.len() };
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let extension = if is_dir {
                    String::new()
                } else {
                    file_path
                        .extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default()
                };

                entries.push(FileEntry {
                    name,
                    path: file_path.to_string_lossy().to_string(),
                    is_dir,
                    size,
                    modified,
                    extension,
                });
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }

    // Sort: directories first, then alphabetical
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

#[command]
fn get_drives() -> Result<Vec<DiskInfo>, String> {
    let mut drives = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let path = Path::new(&drive);
            if !path.exists() {
                continue;
            }

            let mut total: u64 = 0;
            let mut free: u64 = 0;
            let label = get_windows_volume_label(&drive);

            unsafe {
                let path_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
                if GetDiskFreeSpaceExW(
                    path_wide.as_ptr(),
                    std::ptr::null_mut(),
                    &mut total,
                    &mut free,
                ) == 0
                {
                    total = 0;
                    free = 0;
                }
            }

            let fs = get_windows_filesystem(&drive);
            drives.push(DiskInfo {
                name: drive.clone(),
                mount_point: drive.clone(),
                total_space: total,
                available_space: free,
                used_space: total.saturating_sub(free),
                file_system: fs,
                label,
            });
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        if let Ok(info) = get_unix_disk_info("/") {
            drives.push(info);
        }
        if home != "/" {
            if let Ok(info) = get_unix_disk_info(&home) {
                drives.push(info);
            }
        }
        if drives.is_empty() {
            drives.push(DiskInfo {
                name: "Root".to_string(),
                mount_point: "/".to_string(),
                total_space: 0,
                available_space: 0,
                used_space: 0,
                file_system: "ext4".to_string(),
                label: "System".to_string(),
            });
        }
    }

    Ok(drives)
}

#[cfg(target_os = "windows")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailableToCaller: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> i32;

    fn GetVolumeInformationW(
        lpRootPathName: *const u16,
        lpVolumeNameBuffer: *mut u16,
        nVolumeNameSize: u32,
        lpVolumeSerialNumber: *mut u32,
        lpMaximumComponentLength: *mut u32,
        lpFileSystemFlags: *mut u32,
        lpFileSystemNameBuffer: *mut u16,
        nFileSystemNameSize: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
fn get_windows_volume_label(drive: &str) -> String {
    let drive_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = vec![0u16; 128];
    unsafe {
        if GetVolumeInformationW(
            drive_wide.as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            return String::from_utf16_lossy(&buf[..end]);
        }
    }
    String::new()
}

#[cfg(target_os = "windows")]
fn get_windows_filesystem(drive: &str) -> String {
    let drive_wide: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = vec![0u16; 32];
    unsafe {
        if GetVolumeInformationW(
            drive_wide.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            buf.len() as u32,
        ) != 0
        {
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            return String::from_utf16_lossy(&buf[..end]);
        }
    }
    String::new()
}

#[cfg(not(target_os = "windows"))]
fn get_unix_disk_info(path: &str) -> Result<DiskInfo, String> {
    use std::ffi::CString;
    let cpath = CString::new(path).map_err(|e| e.to_string())?;
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(cpath.as_ptr(), &mut stat) != 0 {
            return Err("statvfs failed".to_string());
        }
        let total = stat.f_frsize as u64 * stat.f_blocks;
        let free = stat.f_frsize as u64 * stat.f_bavail;
        Ok(DiskInfo {
            name: path.to_string(),
            mount_point: path.to_string(),
            total_space: total,
            available_space: free,
            used_space: total.saturating_sub(free),
            file_system: "ext4".to_string(),
            label: String::new(),
        })
    }
}

#[command]
fn get_parent_directory(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    match path.parent() {
        Some(parent) => Ok(parent.to_string_lossy().to_string()),
        None => Err("No parent directory".to_string()),
    }
}

#[command]
fn create_directory(state: State<AppState>, path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;
    let mut history = state.undo_history.lock().map_err(|e| e.to_string())?;
    history.push(FileAction {
        kind: ActionKind::Create {
            path: path.clone(),
            is_dir: true,
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    Ok(())
}

#[command]
fn create_file(state: State<AppState>, path: String) -> Result<(), String> {
    if Path::new(&path).exists() {
        return Err("File already exists".to_string());
    }
    fs::write(&path, "").map_err(|e| format!("Failed to create file: {}", e))?;
    let mut history = state.undo_history.lock().map_err(|e| e.to_string())?;
    history.push(FileAction {
        kind: ActionKind::Create {
            path: path.clone(),
            is_dir: false,
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    Ok(())
}

#[command]
fn delete_item(path: String, permanently: bool) -> Result<(), String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if permanently {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("Failed to delete: {}", e))
        } else {
            fs::remove_file(path).map_err(|e| format!("Failed to delete: {}", e))
        }
    } else {
        // Send to trash
        trash::delete(path).map_err(|e| format!("Failed to move to trash: {}", e))
    }
}

#[command]
fn rename_item(state: State<AppState>, old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))?;
    let mut history = state.undo_history.lock().map_err(|e| e.to_string())?;
    history.push(FileAction {
        kind: ActionKind::Rename {
            old_path: old_path.clone(),
            new_path: new_path.clone(),
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    });
    Ok(())
}

#[command]
fn copy_clipboard(state: State<AppState>, paths: Vec<String>) -> Result<(), String> {
    let mut clipboard = state.clipboard.lock().map_err(|e| e.to_string())?;
    let mut action = state.clipboard_action.lock().map_err(|e| e.to_string())?;
    clipboard.clear();
    for p in &paths {
        clipboard.push(PathBuf::from(p));
    }
    *action = "copy".to_string();
    Ok(())
}

#[command]
fn cut_clipboard(state: State<AppState>, paths: Vec<String>) -> Result<(), String> {
    let mut clipboard = state.clipboard.lock().map_err(|e| e.to_string())?;
    let mut action = state.clipboard_action.lock().map_err(|e| e.to_string())?;
    clipboard.clear();
    for p in &paths {
        clipboard.push(PathBuf::from(p));
    }
    *action = "cut".to_string();
    Ok(())
}

#[command]
fn paste_clipboard(state: State<AppState>, dest_dir: String) -> Result<(), String> {
    let clipboard = state.clipboard.lock().map_err(|e| e.to_string())?;
    let action = state.clipboard_action.lock().map_err(|e| e.to_string())?;

    if clipboard.is_empty() {
        return Err("Clipboard is empty".to_string());
    }

    let dest = Path::new(&dest_dir);
    let is_cut = *action == "cut";

    for src_path in clipboard.iter() {
        if let Some(file_name) = src_path.file_name() {
            let dest_path = dest.join(file_name);
            if src_path.is_dir() {
                copy_dir_recursive(src_path, &dest_path)?;
                if is_cut {
                    fs::remove_dir_all(src_path)
                        .map_err(|e| format!("Failed to remove source: {}", e))?;
                }
            } else {
                fs::copy(src_path, &dest_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                if is_cut {
                    fs::remove_file(src_path)
                        .map_err(|e| format!("Failed to remove source: {}", e))?;
                }
            }
        }
    }

    if is_cut {
        drop(clipboard);
        drop(action);
        let mut clipboard = state.clipboard.lock().map_err(|e| e.to_string())?;
        clipboard.clear();
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("Failed to create directory: {}", e))?;

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

#[command]
fn open_in_terminal(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let dir = if p.is_dir() {
        p
    } else {
        p.parent().unwrap_or(p)
    };

    #[cfg(target_os = "windows")]
    {
        // Try Windows Terminal first, fall back to cmd
        let wt = std::process::Command::new("wt")
            .args(["-d", &dir.to_string_lossy()])
            .spawn();
        if wt.is_err() {
            std::process::Command::new("cmd")
                .args([
                    "/C",
                    "start",
                    "cmd",
                    "/K",
                    &format!("cd /d {}", dir.to_string_lossy()),
                ])
                .spawn()
                .map_err(|e| format!("Failed to open terminal: {}", e))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", "Terminal", &dir.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let terms = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "xterm",
        ];
        let mut opened = false;
        for term in &terms {
            if std::process::Command::new(term)
                .arg("--working-directory")
                .arg(&dir.to_string_lossy())
                .spawn()
                .is_ok()
            {
                opened = true;
                break;
            }
        }
        if !opened {
            return Err("No terminal emulator found".to_string());
        }
    }

    Ok(())
}

#[command]
fn get_file_info(path: String) -> Result<FileEntry, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }

    let metadata = fs::metadata(&p).map_err(|e| format!("Failed to get metadata: {}", e))?;

    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let is_dir = metadata.is_dir();
    let size = if is_dir {
        calculate_dir_size(p)
    } else {
        metadata.len()
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let extension = if is_dir {
        String::new()
    } else {
        p.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    Ok(FileEntry {
        name,
        path: p.to_string_lossy().to_string(),
        is_dir,
        size,
        modified,
        extension,
    })
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

#[command]
fn open_file(path: String) -> Result<(), String> {
    opener::open(path).map_err(|e| format!("Failed to open: {}", e))
}

#[command]
fn search_files(directory: String, query: String) -> Result<Vec<FileEntry>, String> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for entry in WalkDir::new(&directory)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .contains(&query_lower)
        })
    {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        results.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            is_dir: entry.file_type().is_dir(),
            size: if entry.file_type().is_dir() {
                0
            } else {
                metadata.len()
            },
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            extension: if entry.file_type().is_dir() {
                String::new()
            } else {
                path.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default()
            },
        });
    }

    Ok(results)
}

#[command]
fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[derive(Debug, Serialize)]
pub struct SpecialDirs {
    pub home: String,
    pub desktop: String,
    pub documents: String,
    pub downloads: String,
    pub pictures: String,
    pub music: String,
    pub videos: String,
}

#[command]
fn get_special_dirs() -> Result<SpecialDirs, String> {
    let home = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE")
            .or_else(|_| {
                std::env::var("HOMEDRIVE")
                    .and_then(|hd| std::env::var("HOMEPATH").map(|hp| format!("{}{}", hd, hp)))
            })
            .unwrap_or_else(|_| String::from("C:\\"))
    } else {
        std::env::var("HOME").unwrap_or_else(|_| String::from("/"))
    };

    let join_path = |parent: &str, child: &str| -> String {
        let sep = if cfg!(target_os = "windows") {
            "\\"
        } else {
            "/"
        };
        if parent.ends_with('\\') || parent.ends_with('/') {
            format!("{}{}", parent, child)
        } else {
            format!("{}{}{}", parent, sep, child)
        }
    };

    Ok(SpecialDirs {
        desktop: join_path(&home, "Desktop"),
        documents: join_path(&home, "Documents"),
        downloads: join_path(&home, "Downloads"),
        pictures: join_path(&home, "Pictures"),
        music: join_path(&home, "Music"),
        videos: join_path(&home, "Videos"),
        home,
    })
}

#[command]
fn get_clipboard_info(state: State<AppState>) -> Result<ClipboardInfo, String> {
    let clipboard = state.clipboard.lock().map_err(|e| e.to_string())?;
    let action = state.clipboard_action.lock().map_err(|e| e.to_string())?;
    Ok(ClipboardInfo {
        paths: clipboard
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        action: action.clone(),
    })
}

#[command]
fn undo_last_action(state: State<AppState>) -> Result<String, String> {
    let mut history = state.undo_history.lock().map_err(|e| e.to_string())?;
    let action = history.pop().ok_or("Nothing to undo".to_string())?;
    match &action.kind {
        ActionKind::Delete { .. } => Err("Cannot undo delete operation".to_string()),
        ActionKind::Rename { old_path, new_path } => {
            fs::rename(new_path, old_path).map_err(|e| format!("Undo rename failed: {}", e))?;
            Ok(format!("Undid rename: restored {old_path}"))
        }
        ActionKind::Create { path, is_dir } => {
            if *is_dir {
                fs::remove_dir_all(path).map_err(|e| format!("Undo create failed: {}", e))?;
            } else {
                fs::remove_file(path).map_err(|e| format!("Undo create failed: {}", e))?;
            }
            Ok(format!("Undid create: removed {path}"))
        }
        ActionKind::Copy { src: _, dest } => {
            let dest_path = Path::new(dest);
            if dest_path.is_dir() {
                fs::remove_dir_all(dest_path).map_err(|e| format!("Undo copy failed: {}", e))?;
            } else {
                fs::remove_file(dest_path).map_err(|e| format!("Undo copy failed: {}", e))?;
            }
            Ok(format!("Undid copy: removed {dest}"))
        }
    }
}

#[command]
fn get_undo_info(state: State<AppState>) -> Result<Option<FileAction>, String> {
    let history = state.undo_history.lock().map_err(|e| e.to_string())?;
    Ok(history.last().cloned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            clipboard: std::sync::Mutex::new(Vec::new()),
            clipboard_action: std::sync::Mutex::new(String::new()),
            undo_history: std::sync::Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            list_directory,
            get_drives,
            get_parent_directory,
            create_directory,
            create_file,
            delete_item,
            rename_item,
            copy_clipboard,
            cut_clipboard,
            paste_clipboard,
            get_file_info,
            open_file,
            open_in_terminal,
            search_files,
            path_exists,
            get_special_dirs,
            get_clipboard_info,
            undo_last_action,
            get_undo_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
