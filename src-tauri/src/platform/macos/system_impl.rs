// platform/macos/system_impl.rs
// macOS system capability implementation.

use crate::core::types::DiskInfo;
use crate::platform::PlatformSystem;
use std::path::{Path, PathBuf};

pub struct SystemImpl;

impl PlatformSystem for SystemImpl {
    fn open_file(&self, path: &Path) -> Result<(), String> {
        opener::open(path).map_err(|e| format!("Failed to open: {}", e))
    }

    fn open_terminal(&self, dir: &Path) -> Result<(), String> {
        let dir_str = dir.to_string_lossy();
        // Prefer iTerm; fall back to Terminal.app
        let iterm = std::process::Command::new("open")
            .args(["-a", "iTerm", &dir_str])
            .spawn();
        if iterm.is_err() {
            std::process::Command::new("open")
                .args(["-a", "Terminal", &dir_str])
                .spawn()
                .map_err(|e| format!("Failed to open terminal: {}", e))?;
        }
        Ok(())
    }

    fn show_in_file_manager(&self, path: &Path) -> Result<(), String> {
        std::process::Command::new("open")
            .args(["-R"])
            .arg(path.as_os_str())
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
        Ok(())
    }

    fn show_properties(&self, path: &Path) -> Result<(), String> {
        // macOS Get Info: ⌘I — reveal in Finder selects it;
        // there's no direct CLI for the Info panel.
        std::process::Command::new("open")
            .args(["-R"])
            .arg(path.as_os_str())
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
        Ok(())
    }

    fn print_file(&self, path: &Path) -> Result<(), String> {
        std::process::Command::new("open")
            .args(["-a", "Preview"])
            .arg(path.as_os_str())
            .spawn()
            .map_err(|e| format!("Print: {}", e))?;
        Ok(())
    }

    fn get_file_icon(&self, path: &Path) -> Result<Vec<u8>, String> {
        get_macos_icon_png(path)
    }

    fn set_auto_start(&self, enabled: bool) -> Result<(), String> {
        let plist_path = home()
            .join("Library")
            .join("LaunchAgents")
            .join("com.files-explorer.desktop.plist");

        if enabled {
            let exe = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.files-explorer.desktop</string>
    <key>ProgramArguments</key>
    <array><string>{}</string></array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><false/>
</dict>
</plist>"#,
                exe
            );
            if let Some(parent) = plist_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&plist_path, plist)
                .map_err(|e| format!("Failed to write plist: {}", e))?;
        } else if plist_path.exists() {
            std::fs::remove_file(&plist_path).ok();
        }
        Ok(())
    }

    fn is_auto_start_enabled(&self) -> bool {
        home()
            .join("Library")
            .join("LaunchAgents")
            .join("com.files-explorer.desktop.plist")
            .exists()
    }

    fn write_clipboard(&self, paths: &[PathBuf]) {
        write_macos_pasteboard(paths);
    }

    fn read_clipboard(&self) -> Option<Vec<PathBuf>> {
        read_macos_pasteboard()
    }

    fn start_native_drag(&self, _paths: &[String]) -> Result<String, String> {
        Err("Native drag only supported on Windows".into())
    }

    fn get_drives(&self) -> Vec<DiskInfo> {
        get_macos_drives()
    }

    fn default_shell(&self) -> String {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into())
    }

    fn is_tray_supported(&self) -> bool {
        true // macOS menu bar extras always work
    }

    fn send_notification(&self, title: &str, body: &str) -> Result<(), String> {
        std::process::Command::new("osascript")
            .args([
                "-e",
                &format!(
                    r#"display notification "{}" with title "{}""#,
                    body.replace('"', "\\\""),
                    title.replace('"', "\\\"")
                ),
            ])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Notification failed: {}", e))
    }
}

impl SystemImpl {
    pub fn instance() -> &'static SystemImpl {
        &SystemImpl
    }
}

fn home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

// ── macOS NSPasteboard ──
fn write_macos_pasteboard(paths: &[PathBuf]) {
    use objc2::{class, msg_send};
    use std::ffi::CString;
    unsafe {
        let pb: *mut objc2::runtime::AnyObject = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: () = msg_send![pb, clearContents];

        let arr: *mut objc2::runtime::AnyObject = msg_send![class!(NSMutableArray), array];
        for p in paths {
            let c_str = CString::new(p.to_string_lossy().as_bytes()).unwrap();
            let nsstr: *mut objc2::runtime::AnyObject =
                msg_send![class!(NSString), stringWithUTF8String: c_str.as_ptr()];
            let url: *mut objc2::runtime::AnyObject =
                msg_send![class!(NSURL), fileURLWithPath: nsstr];
            let _: () = msg_send![arr, addObject: url];
        }
        let _: bool = msg_send![pb, writeObjects: arr];
    }
}

fn read_macos_pasteboard() -> Option<Vec<PathBuf>> {
    use objc2::{class, msg_send};
    unsafe {
        let pb: *mut objc2::runtime::AnyObject = msg_send![class!(NSPasteboard), generalPasteboard];

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
                            paths.push(PathBuf::from(s));
                        }
                    }
                }
                if !paths.is_empty() {
                    return Some(paths);
                }
            }
        }
    }
    // Fallback: plain text via arboard
    if let Ok(mut c) = arboard::Clipboard::new() {
        if let Ok(t) = c.get_text() {
            return Some(t.lines().map(PathBuf::from).collect());
        }
    }
    None
}

// ── macOS drive enumeration ──
fn get_macos_drives() -> Vec<DiskInfo> {
    let mut drives = Vec::new();

    // Root volume always present
    if let Ok(info) = get_unix_disk_info("/") {
        drives.push(info);
    }

    // Enumerate external volumes under /Volumes
    if let Ok(entries) = std::fs::read_dir("/Volumes") {
        let root_dev = std::fs::metadata("/").ok().and_then(|m| {
            use std::os::unix::fs::MetadataExt;
            Some(m.dev())
        });
        for entry in entries.flatten() {
            let vol_path = entry.path();
            if !vol_path.is_dir() {
                continue;
            }
            let vol_str = vol_path.to_string_lossy().to_string();
            // Skip root volume alias (same device ID as /)
            if let Some(rd) = root_dev {
                if let Ok(m) = std::fs::metadata(&vol_str) {
                    use std::os::unix::fs::MetadataExt;
                    if m.dev() == rd {
                        continue;
                    }
                }
            }
            if let Ok(mut info) = get_unix_disk_info(&vol_str) {
                let vol_name = vol_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| vol_str.clone());
                // Skip system-temporary mounts
                let name_lower = vol_name.to_lowercase();
                if name_lower.starts_with("dmg.")
                    || name_lower.starts_with("install ")
                    || name_lower == "preboot"
                    || name_lower == "recovery"
                    || name_lower == "vm"
                {
                    continue;
                }
                info.label = vol_name.clone();
                if info.name == vol_str && vol_name != "/" {
                    info.name = vol_name;
                }
                drives.push(info);
            }
        }
    }

    drives
}

fn get_unix_disk_info(path: &str) -> Result<DiskInfo, String> {
    use std::ffi::{CStr, CString};
    let cpath = CString::new(path).map_err(|e| e.to_string())?;
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(cpath.as_ptr(), &mut stat) != 0 {
            return Err("statvfs failed".to_string());
        }
        let total = stat.f_frsize as u64 * stat.f_blocks as u64;
        let free = stat.f_frsize as u64 * stat.f_bavail as u64;
        let mut sfs: libc::statfs = std::mem::zeroed();
        let file_system = if libc::statfs(cpath.as_ptr(), &mut sfs) == 0 {
            CStr::from_ptr(sfs.f_fstypename.as_ptr())
                .to_string_lossy()
                .to_string()
        } else {
            String::from("unknown")
        };
        Ok(DiskInfo {
            name: path.to_string(),
            mount_point: path.to_string(),
            total_space: total,
            available_space: free,
            used_space: total.saturating_sub(free),
            file_system,
            label: String::new(),
        })
    }
}

// ── macOS file icon via NSWorkspace ──
fn get_macos_icon_png(path: &std::path::Path) -> Result<Vec<u8>, String> {
    use objc2::{class, msg_send};
    use std::ffi::CString;
    unsafe {
        let path_str = path.to_string_lossy();
        let c_path = CString::new(path_str.as_bytes()).map_err(|e| format!("{}", e))?;
        let ns_path: *mut objc2::runtime::AnyObject =
            msg_send![class!(NSString), stringWithUTF8String: c_path.as_ptr()];

        // NSWorkspace.sharedWorkspace.iconForFile:
        let ws: *mut objc2::runtime::AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
        let icon: *mut objc2::runtime::AnyObject = msg_send![ws, iconForFile: ns_path];

        if icon.is_null() {
            return Err("Failed to get icon".into());
        }

        // NSImage → TIFFRepresentation
        let tiff: *mut objc2::runtime::AnyObject = msg_send![icon, TIFFRepresentation];
        if tiff.is_null() {
            return Err("Failed to get TIFF".into());
        }

        // NSBitmapImageRep from TIFF data
        let rep: *mut objc2::runtime::AnyObject =
            msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff];
        if rep.is_null() {
            return Err("Failed to create bitmap rep".into());
        }

        // Convert to PNG
        let props: *mut objc2::runtime::AnyObject = msg_send![class!(NSDictionary), dictionary];
        let png_data: *mut objc2::runtime::AnyObject = msg_send![
            rep,
            representationUsingType: 4u64, // NSPNGFileType = 4
            properties: props
        ];

        if png_data.is_null() {
            return Err("Failed to encode PNG".into());
        }

        let length: usize = msg_send![png_data, length];
        let bytes: *const u8 = msg_send![png_data, bytes];
        if bytes.is_null() || length == 0 {
            return Err("Empty PNG data".into());
        }

        Ok(std::slice::from_raw_parts(bytes, length).to_vec())
    }
}
