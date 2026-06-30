use std::fs;
use std::path::PathBuf;

fn autostart_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(
                PathBuf::from(home).join("Library/LaunchAgents/com.files-explorer.desktop.plist"),
            );
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(
                PathBuf::from(appdata)
                    .join("Microsoft/Windows/Start Menu/Programs/Startup/files-explorer.lnk"),
            );
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home).join(".config/autostart/files-explorer.desktop"));
        }
    }
    None
}

fn current_exe_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn set_auto_start(enabled: bool) -> Result<(), String> {
    let path = autostart_path().ok_or("Cannot determine autostart path")?;

    if enabled {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            let exe = current_exe_path().unwrap_or_default();
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.files-explorer.desktop</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>"#,
                exe
            );
            fs::write(&path, plist).map_err(|e| format!("Failed: {}", e))?;
        }

        #[cfg(target_os = "windows")]
        {
            // Use registry for Windows auto-start (more reliable than shortcut)
            let exe = current_exe_path().unwrap_or_default();
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "FilesExplorer",
                    "/d",
                    &exe,
                    "/f",
                ])
                .output();
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let exe = current_exe_path().unwrap_or_default();
            let desktop = format!(
                r#"[Desktop Entry]
Type=Application
Name=Files Explorer
Exec={}
X-GNOME-Autostart-enabled=true
NoDisplay=false
Hidden=false
"#,
                exe
            );
            fs::write(&path, desktop).map_err(|e| format!("Failed: {}", e))?;
        }
    } else {
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed: {}", e))?;
        }

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("reg")
                .args([
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    "FilesExplorer",
                    "/f",
                ])
                .output();
        }
    }

    Ok(())
}

pub fn is_auto_start_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check registry
        if let Ok(output) = std::process::Command::new("reg")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "FilesExplorer",
            ])
            .output()
        {
            return output.status.success();
        }
        return false;
    }

    #[cfg(not(target_os = "windows"))]
    {
        autostart_path().map(|p| p.exists()).unwrap_or(false)
    }
}
