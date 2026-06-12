use log;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tauri::{command, AppHandle, Emitter, State};
use walkdir::WalkDir;

fn ts_from_metadata(
    metadata: &fs::Metadata,
    getter: fn(&fs::Metadata) -> Result<SystemTime, std::io::Error>,
) -> i64 {
    getter(metadata)
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub created: i64,
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
pub struct SearchProgress {
    pub files: Vec<FileEntry>,
    pub total: u64,
    pub done: bool,
    pub truncated: bool,
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
    search_cancel: Arc<AtomicBool>,
}

#[command]
fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        log::warn!("list_directory: path not found: {}", path);
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
                let modified = ts_from_metadata(&metadata, fs::Metadata::modified);
                let created = ts_from_metadata(&metadata, fs::Metadata::created);
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
                    created,
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

const LIST_BATCH_SIZE: usize = 100;

#[command]
fn list_directory_streamed(app: AppHandle, path: String) -> Result<(), String> {
    let dir = Path::new(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut batch: Vec<FileEntry> = Vec::new();
        if let Ok(rd) = fs::read_dir(&path) {
            for e in rd.flatten() {
                let fp = e.path();
                let md = e.metadata().ok().or_else(|| fs::metadata(&fp).ok());
                let Some(md) = md else { continue };
                batch.push(FileEntry {
                    name: e.file_name().to_string_lossy().to_string(),
                    path: fp.to_string_lossy().to_string(),
                    is_dir: md.is_dir(),
                    size: if md.is_dir() { 0 } else { md.len() },
                    modified: ts_from_metadata(&md, fs::Metadata::modified),
                    created: ts_from_metadata(&md, fs::Metadata::created),
                    extension: if md.is_dir() {
                        String::new()
                    } else {
                        fp.extension()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or_default()
                    },
                });
                if batch.len() >= LIST_BATCH_SIZE {
                    let _ = app2.emit("list-progress", std::mem::take(&mut batch));
                }
            }
        }
        if !batch.is_empty() {
            let _ = app2.emit("list-progress", batch);
        }
        let _ = app2.emit("list-done", true);
    });
    Ok(())
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
    trim_undo_history(&mut history);
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
    trim_undo_history(&mut history);
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
    trim_undo_history(&mut history);
    Ok(())
}

#[command]
fn copy_clipboard(state: State<AppState>, paths: Vec<String>) -> Result<(), String> {
    log::info!("copy_clipboard: {} items", paths.len());
    // 1. Set internal clipboard (guaranteed to work)
    {
        let mut cb = state.clipboard.lock().map_err(|e| e.to_string())?;
        let mut act = state.clipboard_action.lock().map_err(|e| e.to_string())?;
        cb.clear();
        for p in &paths {
            cb.push(PathBuf::from(p));
        }
        *act = "copy".to_string();
    }
    // 2. Best-effort write to system clipboard
    write_to_system_clipboard(&paths);
    Ok(())
}

#[command]
fn cut_clipboard(state: State<AppState>, paths: Vec<String>) -> Result<(), String> {
    log::info!("cut_clipboard: {} items", paths.len());
    // 1. Set internal clipboard
    {
        let mut cb = state.clipboard.lock().map_err(|e| e.to_string())?;
        let mut act = state.clipboard_action.lock().map_err(|e| e.to_string())?;
        cb.clear();
        for p in &paths {
            cb.push(PathBuf::from(p));
        }
        *act = "cut".to_string();
    }
    // 2. Best-effort write to system clipboard
    write_to_system_clipboard(&paths);
    Ok(())
}

fn write_to_system_clipboard(paths: &[String]) {
    #[cfg(target_os = "windows")]
    unsafe {
        extern "system" {
            fn OpenClipboard(h: *mut std::ffi::c_void) -> i32;
            fn EmptyClipboard() -> i32;
            fn SetClipboardData(f: u32, m: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn CloseClipboard() -> i32;
            fn GlobalAlloc(f: u32, b: usize) -> *mut std::ffi::c_void;
            fn GlobalLock(m: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            fn GlobalUnlock(m: *mut std::ffi::c_void) -> i32;
        }
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return;
        }
        EmptyClipboard();
        let mut wide: Vec<u16> = Vec::new();
        for p in paths {
            wide.extend(p.encode_utf16());
            wide.push(0);
        }
        wide.push(0);
        let hdr = std::mem::size_of::<u32>() * 5;
        let sz = hdr + wide.len() * 2;
        let h = GlobalAlloc(2, sz);
        if h.is_null() {
            CloseClipboard();
            return;
        }
        let p = GlobalLock(h) as *mut u8;
        std::ptr::write_bytes(p, 0, sz);
        (p as *mut u32).write(hdr as u32);
        (p.add(16) as *mut u32).write(1); // fWide = TRUE at offset 16
        std::ptr::copy_nonoverlapping(wide.as_ptr(), p.add(hdr) as *mut u16, wide.len());
        GlobalUnlock(h);
        SetClipboardData(15, h);
        CloseClipboard();
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(mut c) = arboard::Clipboard::new() {
            let _ = c.set_text(&paths.join("\n"));
        }
    }
}

fn read_from_system_clipboard() -> Option<Vec<String>> {
    #[cfg(target_os = "windows")]
    unsafe {
        extern "system" {
            fn OpenClipboard(h: *mut std::ffi::c_void) -> i32;
            fn GetClipboardData(f: u32) -> *mut std::ffi::c_void;
            fn CloseClipboard() -> i32;
            fn IsClipboardFormatAvailable(f: u32) -> i32;
            fn DragQueryFileW(h: *mut std::ffi::c_void, i: u32, b: *mut u16, c: u32) -> u32;
        }
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }
        if IsClipboardFormatAvailable(15) == 0 {
            CloseClipboard();
            return None;
        }
        let hd = GetClipboardData(15);
        if hd.is_null() {
            CloseClipboard();
            return None;
        }
        let cnt = DragQueryFileW(hd, 0xFFFFFFFF, std::ptr::null_mut(), 0);
        let mut v = Vec::with_capacity(cnt as usize);
        let mut b = vec![0u16; 32768];
        for i in 0..cnt {
            let l = DragQueryFileW(hd, i, b.as_mut_ptr(), b.len() as u32) as usize;
            if l > 0 {
                v.push(String::from_utf16_lossy(&b[..l]));
            }
        }
        CloseClipboard();
        return Some(v);
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(mut c) = arboard::Clipboard::new() {
            if let Ok(t) = c.get_text() {
                return Some(t.lines().map(|s| s.to_string()).collect());
            }
        }
        None
    }
}

const PASTE_CONFLICT_SUFFIX: &str = " - Copy";

#[command]
fn paste_clipboard(state: State<AppState>, dest_dir: String) -> Result<(), String> {
    log::info!("paste_clipboard: to {}", dest_dir);

    // Always try system clipboard FIRST
    if let Some(sys_paths) = read_from_system_clipboard() {
        if !sys_paths.is_empty() {
            // Check if internal clipboard says "cut" (for cut-paste delete)
            let is_cut = {
                state
                    .clipboard_action
                    .lock()
                    .map(|a| *a == "cut")
                    .unwrap_or(false)
            };
            let dest = Path::new(&dest_dir);
            for src_str in &sys_paths {
                let src = Path::new(src_str);
                if !src.exists() {
                    continue;
                }
                if let Some(n) = src.file_name() {
                    let dp = resolve_paste_conflict(&dest.join(n));
                    if src.is_dir() {
                        copy_dir_recursive(src, &dp)?;
                        if is_cut {
                            fs::remove_dir_all(src).map_err(|e| format!("Remove failed: {}", e))?;
                        }
                    } else {
                        fs::copy(src, &dp).map_err(|e| format!("Copy failed: {}", e))?;
                        if is_cut {
                            fs::remove_file(src).map_err(|e| format!("Remove failed: {}", e))?;
                        }
                    }
                    let mut h = state.undo_history.lock().map_err(|e| e.to_string())?;
                    h.push(FileAction {
                        kind: ActionKind::Copy {
                            src: src_str.clone(),
                            dest: dp.to_string_lossy().to_string(),
                        },
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    });
                    trim_undo_history(&mut h);
                }
            }
            if is_cut {
                let _ = state.clipboard.lock().map(|mut c| c.clear());
                let _ = state
                    .clipboard_action
                    .lock()
                    .map(|mut a| *a = String::new());
            }
            return Ok(());
        }
    }

    // Fall back to internal clipboard
    let clipboard = state.clipboard.lock().map_err(|e| e.to_string())?;
    let action = state.clipboard_action.lock().map_err(|e| e.to_string())?;
    if clipboard.is_empty() {
        return Err("Clipboard is empty".to_string());
    }

    let dest = Path::new(&dest_dir);
    let is_cut = *action == "cut";
    for src_path in clipboard.iter() {
        if let Some(file_name) = src_path.file_name() {
            let dest_path = resolve_paste_conflict(&dest.join(file_name));
            if src_path.is_dir() {
                copy_dir_recursive(src_path, &dest_path)?;
                if is_cut {
                    fs::remove_dir_all(src_path).map_err(|e| format!("Failed to remove: {}", e))?;
                }
            } else {
                fs::copy(src_path, &dest_path).map_err(|e| format!("Failed to copy: {}", e))?;
                if is_cut {
                    fs::remove_file(src_path).map_err(|e| format!("Failed to remove: {}", e))?;
                }
            }
            if !is_cut {
                let mut h = state.undo_history.lock().map_err(|e| e.to_string())?;
                h.push(FileAction {
                    kind: ActionKind::Copy {
                        src: src_path.to_string_lossy().to_string(),
                        dest: dest_path.to_string_lossy().to_string(),
                    },
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                });
                trim_undo_history(&mut h);
            }
        }
    }
    if is_cut {
        drop(clipboard);
        drop(action);
        state.clipboard.lock().map_err(|e| e.to_string())?.clear();
    }
    Ok(())
}

/// Resolve name conflict by appending " - Copy", " - Copy (2)", etc.
fn resolve_paste_conflict(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut counter: u32 = 1;
    loop {
        let new_name = if counter == 1 {
            if ext.is_empty() {
                format!("{}{}", stem, PASTE_CONFLICT_SUFFIX)
            } else {
                format!("{}{}.{}", stem, PASTE_CONFLICT_SUFFIX, ext)
            }
        } else {
            if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            }
        };
        let candidate = parent.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
        if counter > 999 {
            break;
        }
    }
    // Fallback: use timestamp
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    parent.join(format!("{}_{}.{}", stem, ts, ext))
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
    let size = if is_dir { 0 } else { metadata.len() };
    let modified = ts_from_metadata(&metadata, fs::Metadata::modified);
    let created = ts_from_metadata(&metadata, fs::Metadata::created);
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
        created,
        extension,
    })
}

// (calculate_dir_size removed - use async walkdir for large dirs if needed)

#[command]
fn open_file(path: String) -> Result<(), String> {
    opener::open(path).map_err(|e| format!("Failed to open: {}", e))
}

// ── Wildcard / size / OR search ──

/// Match a filename against a glob pattern (* = any chars, ? = one char)
fn wildcard_match(pattern: &str, filename: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let f: Vec<char> = filename.chars().collect();
    fn rec(p: &[char], f: &[char]) -> bool {
        match (p.is_empty(), f.is_empty()) {
            (true, true) => true,
            (true, false) => false,
            (false, true) => p.iter().all(|&c| c == '*'),
            (false, false) => {
                if p[0] == '*' {
                    rec(&p[1..], f) || rec(p, &f[1..])
                } else if p[0] == '?' || p[0].to_ascii_lowercase() == f[0].to_ascii_lowercase() {
                    rec(&p[1..], &f[1..])
                } else {
                    false
                }
            }
        }
    }
    rec(&p, &f)
}

/// Parse a size filter like ">10MB" or "<1GB" into (operator, bytes)
fn parse_size_filter(s: &str) -> Option<(char, u64)> {
    let s = s.trim();
    let op = s.chars().next()?;
    if op != '>' && op != '<' {
        return None;
    }
    let rest = s[1..].trim();
    // Extract numeric part
    let num_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    if num_end == 0 {
        return None;
    }
    let num: f64 = rest[..num_end].parse().ok()?;
    let unit = rest[num_end..].trim().to_lowercase();
    let multiplier = match unit.as_str() {
        "b" | "" => 1.0,
        "k" | "kb" => 1024.0,
        "m" | "mb" => 1024.0 * 1024.0,
        "g" | "gb" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((op, (num * multiplier) as u64))
}

/// Check whether a single condition matches a file
fn condition_matches_file(condition: &str, name: &str, size: u64) -> bool {
    let cond = condition.trim();
    if cond.is_empty() {
        return false;
    }
    // Size filter?
    if let Some((op, threshold)) = parse_size_filter(cond) {
        return match op {
            '>' => size > threshold,
            '<' => size < threshold,
            _ => false,
        };
    }
    // Wildcard pattern? (contains * or ?)
    if cond.contains('*') || cond.contains('?') {
        return wildcard_match(cond, name);
    }
    // Plain text: substring match (case-insensitive)
    name.to_lowercase().contains(&cond.to_lowercase())
}

#[command]
fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}

const SEARCH_MAX_RESULTS: u64 = 2000;
const SEARCH_BATCH_SIZE: usize = 500;

#[command]
fn move_files(paths: Vec<String>, dest_dir: String, copy: bool) -> Result<(), String> {
    log::info!(
        "move_files: {} items to {} (copy={})",
        paths.len(),
        dest_dir,
        copy
    );
    let dest = Path::new(&dest_dir);
    for src_str in &paths {
        let src = Path::new(src_str);
        if !src.exists() {
            continue;
        }
        let file_name = match src.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        let dest_path = dest.join(&file_name);
        if dest_path.exists() {
            return Err(format!("Target already exists: {}", file_name));
        }
        // Try fast rename first (same filesystem)
        if !copy && std::fs::rename(src, &dest_path).is_ok() {
            continue;
        }
        // Cross-device or copy: fall back to copy-then-delete
        if src.is_dir() {
            copy_dir_recursive(src, &dest_path)?;
        } else {
            std::fs::copy(src, &dest_path).map_err(|e| format!("Failed to copy: {}", e))?;
        }
        if !copy {
            if src.is_dir() {
                std::fs::remove_dir_all(src)
                    .map_err(|e| format!("Failed to remove source: {}", e))?;
            } else {
                std::fs::remove_file(src).map_err(|e| format!("Failed to remove source: {}", e))?;
            }
        }
    }
    Ok(())
}

/// Run search in a dedicated thread so it never blocks the UI
#[command]
fn search_files(
    app: AppHandle,
    state: State<AppState>,
    directory: String,
    query: String,
) -> Result<(), String> {
    // Reset cancel flag
    state.search_cancel.store(false, Ordering::SeqCst);
    let cancel = state.search_cancel.clone();

    let conditions: Vec<String> = query
        .split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if conditions.is_empty() {
        let _ = app.emit(
            "search-progress",
            SearchProgress {
                files: vec![],
                total: 0,
                done: true,
                truncated: false,
            },
        );
        return Ok(());
    }

    let dir = directory.clone();
    let app_clone = app.clone();

    // Spawn on a dedicated OS thread — never blocks Tauri's IPC thread
    std::thread::spawn(move || {
        let mut batch: Vec<FileEntry> = Vec::new();
        let mut total: u64 = 0;
        let mut truncated = false;

        for entry in WalkDir::new(&dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Check cancellation (thread-safe AtomicBool)
            if cancel.load(Ordering::SeqCst) {
                let files = std::mem::take(&mut batch);
                let _ = app_clone.emit(
                    "search-progress",
                    SearchProgress {
                        files,
                        total,
                        done: true,
                        truncated: false,
                    },
                );
                return;
            }

            if total >= SEARCH_MAX_RESULTS {
                truncated = true;
                break;
            }

            let path = entry.path().to_path_buf();
            let file_name = entry.file_name().to_string_lossy().to_string();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let file_size = if entry.file_type().is_dir() {
                0
            } else {
                metadata.len()
            };
            let is_dir = entry.file_type().is_dir();
            let modified = ts_from_metadata(&metadata, fs::Metadata::modified);
            let created = ts_from_metadata(&metadata, fs::Metadata::created);
            let extension = if is_dir {
                String::new()
            } else {
                path.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default()
            };

            let matched = conditions
                .iter()
                .any(|cond| condition_matches_file(cond, &file_name, file_size));
            if !matched {
                continue;
            }

            total += 1;
            batch.push(FileEntry {
                name: file_name,
                path: path.to_string_lossy().to_string(),
                is_dir,
                size: file_size,
                modified,
                created,
                extension,
            });

            if batch.len() >= SEARCH_BATCH_SIZE {
                let files = std::mem::take(&mut batch);
                let payload = SearchProgress {
                    files,
                    total,
                    done: false,
                    truncated: false,
                };
                if app_clone.emit("search-progress", payload).is_err() {
                    break;
                }
            }
        }

        let files = std::mem::take(&mut batch);
        let _ = app_clone.emit(
            "search-progress",
            SearchProgress {
                files,
                total,
                done: true,
                truncated,
            },
        );
    });

    Ok(())
}

#[command]
fn cancel_search(state: State<AppState>) -> Result<(), String> {
    state.search_cancel.store(true, Ordering::SeqCst);
    Ok(())
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

const MAX_UNDO_HISTORY: usize = 50;

#[command]
fn get_clipboard_info(state: State<AppState>) -> Result<ClipboardInfo, String> {
    if let Some(p) = read_from_system_clipboard() {
        if !p.is_empty() {
            return Ok(ClipboardInfo {
                paths: p,
                action: "copy".into(),
            });
        }
    }
    let c = state.clipboard.lock().map_err(|e| e.to_string())?;
    let a = state.clipboard_action.lock().map_err(|e| e.to_string())?;
    Ok(ClipboardInfo {
        paths: c.iter().map(|x| x.to_string_lossy().to_string()).collect(),
        action: a.clone(),
    })
}

#[command]
fn undo_last_action(state: State<AppState>) -> Result<String, String> {
    let mut history = state.undo_history.lock().map_err(|e| e.to_string())?;
    let action = history.pop().ok_or("Nothing to undo".to_string())?;
    match &action.kind {
        ActionKind::Delete => Err("Cannot undo delete operation".to_string()),
        ActionKind::Rename { old_path, new_path } => {
            if !Path::new(new_path).exists() {
                return Err(format!("Cannot undo: {new_path} no longer exists"));
            }
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
        ActionKind::Copy { src, dest } => {
            if !Path::new(dest).exists() {
                return Err(format!("Cannot undo: {dest} no longer exists"));
            }
            let dest_path = Path::new(dest);
            if dest_path.is_dir() {
                fs::remove_dir_all(dest_path).map_err(|e| format!("Undo copy failed: {}", e))?;
            } else {
                fs::remove_file(dest_path).map_err(|e| format!("Undo copy failed: {}", e))?;
            }
            Ok(format!("Undid copy of {src}: removed {dest}"))
        }
    }
}

/// Trim undo history to MAX_UNDO_HISTORY entries
fn trim_undo_history(history: &mut Vec<FileAction>) {
    if history.len() > MAX_UNDO_HISTORY {
        let excess = history.len() - MAX_UNDO_HISTORY;
        history.drain(0..excess);
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
            search_cancel: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            list_directory,
            list_directory_streamed,
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
            cancel_search,
            move_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
