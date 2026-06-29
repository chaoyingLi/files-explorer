use crate::error::{op_err, FsError, FsResult};
use crate::operations::{copy_dir_recursive, resolve_paste_conflict, trim_undo_history};
use crate::state::AppState;
use crate::types::{ActionKind, ClipboardInfo, FileAction};
use log;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

pub fn copy_clipboard(state: State<AppState>, paths: Vec<String>) -> FsResult<()> {
    log::info!("copy_clipboard: {} items", paths.len());
    // 1. Set internal clipboard (guaranteed to work)
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|e| FsError::Other(e.to_string()))?;
        inner.clipboard.clear();
        for p in &paths {
            inner.clipboard.push(PathBuf::from(p));
        }
        inner.clipboard_action = "copy".to_string();
    }
    // 2. Best-effort write to system clipboard
    write_to_system_clipboard(&paths);
    Ok(())
}

pub fn cut_clipboard(state: State<AppState>, paths: Vec<String>) -> FsResult<()> {
    log::info!("cut_clipboard: {} items", paths.len());
    // 1. Set internal clipboard
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|e| FsError::Other(e.to_string()))?;
        inner.clipboard.clear();
        for p in &paths {
            inner.clipboard.push(PathBuf::from(p));
        }
        inner.clipboard_action = "cut".to_string();
    }
    // 2. Best-effort write to system clipboard
    write_to_system_clipboard(&paths);
    Ok(())
}

// ── macOS native pasteboard (NSPasteboard) for proper file copy/paste ──

#[cfg(target_os = "macos")]
pub fn write_to_macos_pasteboard(paths: &[String]) {
    use objc2::{class, msg_send};
    use std::ffi::CString;
    unsafe {
        let pb: *mut objc2::runtime::AnyObject = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: () = msg_send![pb, clearContents];

        // Build an NSArray of file URLs
        let arr: *mut objc2::runtime::AnyObject = msg_send![class!(NSMutableArray), array];
        for p in paths {
            let c_str = CString::new(p.as_str()).unwrap();
            let nsstr: *mut objc2::runtime::AnyObject =
                msg_send![class!(NSString), stringWithUTF8String: c_str.as_ptr()];
            let url: *mut objc2::runtime::AnyObject =
                msg_send![class!(NSURL), fileURLWithPath: nsstr];
            let _: () = msg_send![arr, addObject: url];
        }
        let _: bool = msg_send![pb, writeObjects: arr];
    }
}

#[cfg(target_os = "macos")]
pub fn read_from_macos_pasteboard() -> Option<Vec<String>> {
    use objc2::{class, msg_send};
    unsafe {
        let pb: *mut objc2::runtime::AnyObject = msg_send![class!(NSPasteboard), generalPasteboard];

        // Read file URLs from pasteboard
        let url_class: *mut objc2::runtime::AnyObject = msg_send![class!(NSURL), class];
        let classes: *mut objc2::runtime::AnyObject =
            msg_send![class!(NSArray), arrayWithObject: url_class];
        let options: *mut objc2::runtime::AnyObject = msg_send![class!(NSDictionary), dictionary];
        let urls: *mut objc2::runtime::AnyObject =
            msg_send![pb, readObjectsForClasses: classes, options: options];

        if !urls.is_null() {
            let count: usize = msg_send![urls, count];
            if count > 0 {
                let mut paths = Vec::with_capacity(count);
                for i in 0..count {
                    let url: *mut objc2::runtime::AnyObject = msg_send![urls, objectAtIndex: i];
                    let path_obj: *mut objc2::runtime::AnyObject = msg_send![url, path];
                    let cstr: *const std::ffi::c_char =
                        msg_send![path_obj, fileSystemRepresentation];
                    if !cstr.is_null() {
                        if let Ok(s) = std::ffi::CStr::from_ptr(cstr).to_str() {
                            paths.push(s.to_string());
                        }
                    }
                }
                if !paths.is_empty() {
                    return Some(paths);
                }
            }
        }
    }
    // Fallback: try reading plain text
    if let Ok(mut c) = arboard::Clipboard::new() {
        if let Ok(t) = c.get_text() {
            return Some(t.lines().map(|s| s.to_string()).collect());
        }
    }
    None
}

pub fn write_to_system_clipboard(paths: &[String]) {
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
            fn GlobalFree(h: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        let mut retries = 3;
        while retries > 0 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            retries -= 1;
        }
        if retries == 0 {
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
        // Bug 6 fix: on failure, free the allocated memory before closing
        if SetClipboardData(15, h).is_null() {
            GlobalFree(h);
        }
        CloseClipboard();
    }
    #[cfg(target_os = "macos")]
    {
        write_to_macos_pasteboard(paths);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(mut c) = arboard::Clipboard::new() {
            let _ = c.set_text(&paths.join("\n"));
        }
    }
}

pub fn read_from_system_clipboard() -> Option<Vec<String>> {
    #[cfg(target_os = "windows")]
    unsafe {
        extern "system" {
            fn OpenClipboard(h: *mut std::ffi::c_void) -> i32;
            fn GetClipboardData(f: u32) -> *mut std::ffi::c_void;
            fn CloseClipboard() -> i32;
            fn IsClipboardFormatAvailable(f: u32) -> i32;
            fn DragQueryFileW(h: *mut std::ffi::c_void, i: u32, b: *mut u16, c: u32) -> u32;
        }
        let mut retries = 3;
        while retries > 0 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            retries -= 1;
        }
        if retries == 0 {
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
    #[cfg(target_os = "macos")]
    {
        read_from_macos_pasteboard()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(mut c) = arboard::Clipboard::new() {
            if let Ok(t) = c.get_text() {
                return Some(t.lines().map(|s| s.to_string()).collect());
            }
        }
        None
    }
}

pub fn paste_clipboard(state: State<AppState>, dest_dir: String) -> FsResult<()> {
    log::info!("paste_clipboard: to {}", dest_dir);

    // Always try system clipboard FIRST
    if let Some(sys_paths) = read_from_system_clipboard() {
        if !sys_paths.is_empty() {
            // Check if internal clipboard says "cut" (for cut-paste delete)
            let is_cut = {
                state
                    .inner
                    .lock()
                    .ok()
                    .map(|i| i.clipboard_action == "cut")
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
                            fs::remove_dir_all(src).map_err(|e| op_err("Remove failed", e))?;
                        }
                    } else {
                        fs::copy(src, &dp).map_err(|e| op_err("Copy failed", e))?;
                        if is_cut {
                            fs::remove_file(src).map_err(|e| op_err("Remove failed", e))?;
                        }
                    }
                    let mut inner = state
                        .inner
                        .lock()
                        .map_err(|e| FsError::Other(e.to_string()))?;
                    inner.undo_history.push(FileAction {
                        kind: ActionKind::Copy {
                            src: src_str.clone(),
                            dest: dp.to_string_lossy().to_string(),
                            was_cut: is_cut,
                        },
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    });
                    trim_undo_history(&mut inner.undo_history);
                }
            }
            if is_cut {
                let _ = state.inner.lock().map(|mut i| {
                    i.clipboard.clear();
                    i.clipboard_action.clear();
                });
            }
            return Ok(());
        }
    }

    // Fall back to internal clipboard
    let inner = state
        .inner
        .lock()
        .map_err(|e| FsError::Other(e.to_string()))?;
    if inner.clipboard.is_empty() {
        return Err(FsError::Other("Clipboard is empty".into()));
    }

    // Collect paths to avoid holding a borrow on inner during mutation
    let clipboard_paths: Vec<PathBuf> = inner.clipboard.clone();
    let dest = Path::new(&dest_dir);
    let is_cut = inner.clipboard_action == "cut";

    drop(inner);

    for src_path in &clipboard_paths {
        if let Some(file_name) = src_path.file_name() {
            let dest_path = resolve_paste_conflict(&dest.join(file_name));
            if src_path.is_dir() {
                copy_dir_recursive(src_path, &dest_path)?;
                if is_cut {
                    fs::remove_dir_all(src_path).map_err(|e| op_err("Failed to remove", e))?;
                }
            } else {
                fs::copy(src_path, &dest_path).map_err(|e| op_err("Failed to copy", e))?;
                if is_cut {
                    fs::remove_file(src_path).map_err(|e| op_err("Failed to remove", e))?;
                }
            }
            if !is_cut {
                let mut inner = state
                    .inner
                    .lock()
                    .map_err(|e| FsError::Other(e.to_string()))?;
                inner.undo_history.push(FileAction {
                    kind: ActionKind::Copy {
                        src: src_path.to_string_lossy().to_string(),
                        dest: dest_path.to_string_lossy().to_string(),
                        was_cut: false,
                    },
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64,
                });
                trim_undo_history(&mut inner.undo_history);
            }
        }
    }
    if is_cut {
        let _ = state.inner.lock().map(|mut i| {
            i.clipboard.clear();
            i.clipboard_action.clear();
        });
    }
    Ok(())
}

pub fn get_clipboard_info(state: State<AppState>) -> FsResult<ClipboardInfo> {
    if let Some(p) = read_from_system_clipboard() {
        if !p.is_empty() {
            return Ok(ClipboardInfo {
                paths: p,
                action: "copy".into(),
            });
        }
    }
    let inner = state
        .inner
        .lock()
        .map_err(|e| FsError::Other(e.to_string()))?;
    Ok(ClipboardInfo {
        paths: inner
            .clipboard
            .iter()
            .map(|x| x.to_string_lossy().to_string())
            .collect(),
        action: inner.clipboard_action.clone(),
    })
}
