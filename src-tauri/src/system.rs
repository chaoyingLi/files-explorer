use crate::native_drag;
use log;
use std::path::Path;

pub fn open_in_terminal(path: String) -> Result<(), String> {
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
        // Prefer iTerm2, fall back to Terminal.app
        let iterm = std::process::Command::new("open")
            .args(["-a", "iTerm", &dir.to_string_lossy()])
            .spawn();
        if iterm.is_err() {
            std::process::Command::new("open")
                .args(["-a", "Terminal", &dir.to_string_lossy()])
                .spawn()
                .map_err(|e| format!("Failed to open terminal: {}", e))?;
        }
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

pub fn start_native_drag_cmd(paths: Vec<String>) -> Result<String, String> {
    native_drag::start_native_drag(&paths)
}

pub fn open_file(path: String) -> Result<(), String> {
    opener::open(path).map_err(|e| format!("Failed to open: {}", e))
}

pub fn get_file_base64(path: String) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("Read failed: {}", e))?;
    // Limit to 2MB to avoid OOM on huge files
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("File too large for preview".to_string());
    }
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => "image/png",
    };
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(serde_json::json!({
        "mime": mime,
        "data": b64
    }))
}

pub fn show_in_explorer(path: String) -> Result<(), String> {
    log::info!("show_in_explorer: {}", path);
    #[cfg(target_os = "windows")]
    {
        // Parse /select,<path> — comma must be in same arg
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| format!("Failed: {}", e))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed: {}", e))?;
        }
    }
    Ok(())
}

pub fn show_file_properties(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn SHObjectProperties(
                hwnd: *mut std::ffi::c_void,
                dwType: u32,
                pszName: *const u16,
                pszParameters: *const u16,
            ) -> i32;
        }
        const SHOP_FILEPATH: u32 = 0x2;
        let fp: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let ret = SHObjectProperties(
                std::ptr::null_mut(),
                SHOP_FILEPATH,
                fp.as_ptr(),
                std::ptr::null(),
            );
            // Returns TRUE (1) on success, FALSE (0) on failure
            if ret == 0 {
                return Err(format!("SHObjectProperties failed with code: {}", ret));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Reveal in Finder — standard macOS behaviour
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| format!("Failed to show in Finder: {}", e))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Linux: try various file managers to show properties, fallback to terminal `file` command
        #[cfg(target_os = "linux")]
        {
            let escaped = path.replace('"', "\\\"");
            if std::process::Command::new("gio")
                .args(["open", &format!("file:///{}", path.trim_start_matches('/'))])
                .spawn()
                .is_err()
            {
                // Fallback: show file info in terminal
                let _ = std::process::Command::new("sh")
                    .args(["-c", &format!("file \"{}\" 2>/dev/null", escaped)])
                    .spawn();
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            if let Some(parent) = Path::new(&path).parent() {
                let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
            }
        }
    }

    Ok(())
}
