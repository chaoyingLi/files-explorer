// platform/linux/system_impl.rs
// Linux system capability implementation.

use crate::core::types::DiskInfo;
use crate::platform::PlatformSystem;
use std::path::{Path, PathBuf};

pub struct SystemImpl;

impl PlatformSystem for SystemImpl {
    fn open_file(&self, path: &Path) -> Result<(), String> {
        opener::open(path).map_err(|e| format!("Failed to open: {}", e))
    }

    fn open_terminal(&self, dir: &Path) -> Result<(), String> {
        let terms = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "xterm",
        ];
        for term in &terms {
            // Some terminals use --working-directory, others use -d
            let r1 = std::process::Command::new(term)
                .arg("--working-directory")
                .arg(dir.as_os_str())
                .spawn();
            if r1.is_ok() {
                return Ok(());
            }
            let r2 = std::process::Command::new(term)
                .arg("-d")
                .arg(dir.as_os_str())
                .spawn();
            if r2.is_ok() {
                return Ok(());
            }
        }
        Err("No terminal emulator found. Install gnome-terminal, konsole, or xterm.".into())
    }

    fn show_in_file_manager(&self, path: &Path) -> Result<(), String> {
        // Try FreeDesktop FileManager1 DBus API first (selects/highlights file)
        if let Ok(abs) = path.canonicalize() {
            let uri = format!("file://{}", abs.display());
            let dbus = std::process::Command::new("dbus-send")
                .args([
                    "--session",
                    "--dest=org.freedesktop.FileManager1",
                    "--type=method_call",
                    "/org/freedesktop/FileManager1",
                    "org.freedesktop.FileManager1.ShowItems",
                    &format!("array:string:{}", uri),
                    "string:",
                ])
                .spawn();
            if dbus.is_ok() {
                return Ok(());
            }
        }
        // Fallback: open parent directory
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {}", e))?;
        }
        Ok(())
    }

    fn show_properties(&self, path: &Path) -> Result<(), String> {
        let escaped = path.to_string_lossy().replace('"', "\\\"");
        if std::process::Command::new("gio")
            .args([
                "open",
                &format!("file://{}", path.to_string_lossy().trim_start_matches('/')),
            ])
            .spawn()
            .is_err()
        {
            let _ = std::process::Command::new("sh")
                .args(["-c", &format!("file \"{}\" 2>/dev/null", escaped)])
                .spawn();
        }
        Ok(())
    }

    fn print_file(&self, path: &Path) -> Result<(), String> {
        std::process::Command::new("lp")
            .arg(path.as_os_str())
            .spawn()
            .map_err(|e| format!("Print failed: {}", e))?;
        Ok(())
    }

    fn get_file_icon(&self, path: &Path) -> Result<Vec<u8>, String> {
        // Try gio info to find thumbnail, then read it
        let output = std::process::Command::new("gio")
            .args(["info", "-a", "thumbnail::path"])
            .arg(path.as_os_str())
            .output()
            .map_err(|e| format!("gio: {}", e))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(thumb) = line.strip_prefix("thumbnail::path: ") {
                    if let Ok(data) = std::fs::read(thumb.trim()) {
                        return Ok(data);
                    }
                }
            }
        }
        Err("File icon not available. Install gvfs to enable thumbnail support.".into())
    }

    fn set_auto_start(&self, enabled: bool) -> Result<(), String> {
        let desktop_path = home()
            .join(".config")
            .join("autostart")
            .join("files-explorer.desktop");

        if enabled {
            let exe = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let content = format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Name=Files Explorer\n\
                 Exec={}\n\
                 X-GNOME-Autostart-enabled=true\n\
                 NoDisplay=false\n\
                 Hidden=false\n",
                exe
            );
            if let Some(parent) = desktop_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&desktop_path, content)
                .map_err(|e| format!("Failed to write desktop entry: {}", e))?;
        } else if desktop_path.exists() {
            std::fs::remove_file(&desktop_path).ok();
        }
        Ok(())
    }

    fn is_auto_start_enabled(&self) -> bool {
        home()
            .join(".config")
            .join("autostart")
            .join("files-explorer.desktop")
            .exists()
    }

    fn write_clipboard(&self, paths: &[PathBuf]) {
        if let Ok(mut c) = arboard::Clipboard::new() {
            let joined: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let _ = c.set_text(&joined.join("\n"));
        }
    }

    fn read_clipboard(&self) -> Option<Vec<PathBuf>> {
        if let Ok(mut c) = arboard::Clipboard::new() {
            if let Ok(t) = c.get_text() {
                return Some(t.lines().map(PathBuf::from).collect());
            }
        }
        None
    }

    fn start_native_drag(&self, _paths: &[String]) -> Result<String, String> {
        Err("Native drag only supported on Windows".into())
    }

    fn get_drives(&self) -> Vec<DiskInfo> {
        get_linux_drives()
    }

    fn is_tray_supported(&self) -> bool {
        // Wayland compositors may not support legacy X11 tray protocol.
        // Check XDG_SESSION_TYPE; if wayland, tray may fail.
        std::env::var("XDG_SESSION_TYPE")
            .map(|t| t != "wayland")
            .unwrap_or(true)
    }

    fn send_notification(&self, title: &str, body: &str) -> Result<(), String> {
        std::process::Command::new("notify-send")
            .args([title, body])
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

// ── Linux drive enumeration ──
fn get_linux_drives() -> Vec<DiskInfo> {
    let mut drives = Vec::new();

    if let Ok(info) = get_unix_disk_info("/") {
        drives.push(info);
    }

    let home = home();
    if home != PathBuf::from("/") {
        if let Ok(info) = get_unix_disk_info(&home.to_string_lossy()) {
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
            file_system: "unknown".to_string(),
            label: "System".to_string(),
        });
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
