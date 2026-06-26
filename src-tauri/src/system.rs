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
            if ret == 0 {
                return Err(format!("SHObjectProperties failed with code: {}", ret));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| format!("Failed to show in Finder: {}", e))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        #[cfg(target_os = "linux")]
        {
            let escaped = path.replace('"', "\\\"");
            if std::process::Command::new("gio")
                .args(["open", &format!("file:///{}", path.trim_start_matches('/'))])
                .spawn()
                .is_err()
            {
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

// ── Get native OS file icon (base64-encoded PNG) ──

#[cfg(target_os = "windows")]
pub fn get_file_icon(path: String) -> Result<String, String> {
    use base64::Engine;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    const SHGFI_ICON: u32 = 0x000000100;
    const SHGFI_LARGEICON: u32 = 0x000000000;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x000000010;
    const ICON_SIZE: i32 = 32;

    #[repr(C)]
    struct SHFILEINFOW {
        hIcon: *mut std::ffi::c_void,
        iIcon: i32,
        dwAttributes: u32,
        szDisplayName: [u16; 260],
        szTypeName: [u16; 80],
    }

    #[repr(C)]
    struct BITMAPINFOHEADER {
        biSize: u32,
        biWidth: i32,
        biHeight: i32,
        biPlanes: u16,
        biBitCount: u16,
        biCompression: u32,
        biSizeImage: u32,
        biXPelsPerMeter: i32,
        biYPelsPerMeter: i32,
        biClrUsed: u32,
        biClrImportant: u32,
    }

    extern "system" {
        fn SHGetFileInfoW(
            pszPath: *const u16,
            dwFileAttributes: u32,
            psfi: *mut SHFILEINFOW,
            cbFileInfo: u32,
            uFlags: u32,
        ) -> isize;
        fn DestroyIcon(hIcon: *mut std::ffi::c_void) -> i32;
        fn GetDC(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn ReleaseDC(hWnd: *mut std::ffi::c_void, hDC: *mut std::ffi::c_void) -> i32;
        fn CreateCompatibleDC(hdc: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn DeleteDC(hdc: *mut std::ffi::c_void) -> i32;
        fn CreateDIBSection(
            hdc: *mut std::ffi::c_void,
            pbmi: *const std::ffi::c_void,
            usage: u32,
            ppvBits: *mut *mut u8,
            hSection: *mut std::ffi::c_void,
            offset: u32,
        ) -> *mut std::ffi::c_void;
        fn SelectObject(
            hdc: *mut std::ffi::c_void,
            h: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn DeleteObject(h: *mut std::ffi::c_void) -> i32;
        fn DrawIconEx(
            hdc: *mut std::ffi::c_void,
            xLeft: i32,
            yTop: i32,
            hIcon: *mut std::ffi::c_void,
            cxWidth: i32,
            cyWidth: i32,
            istepIfAniCur: u32,
            hbrFlickerFreeDraw: *mut std::ffi::c_void,
            diFlags: u32,
        ) -> i32;
    }

    unsafe {
        let path_wide: Vec<u16> = OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let ret = SHGetFileInfoW(
            path_wide.as_ptr(),
            0,
            &mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
        );

        if ret == 0 || shfi.hIcon.is_null() {
            return Err("SHGetFileInfoW failed".to_string());
        }

        let hicon = shfi.hIcon;
        let screen_dc = GetDC(std::ptr::null_mut());
        if screen_dc.is_null() {
            DestroyIcon(hicon);
            return Err("GetDC failed".to_string());
        }

        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.is_null() {
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            DestroyIcon(hicon);
            return Err("CreateCompatibleDC failed".to_string());
        }

        let w = ICON_SIZE;
        let h = ICON_SIZE;
        let bpp: u16 = 32;
        let row_bytes = ((w as u32 * bpp as u32 + 31) / 32) * 4;
        let pixel_size = row_bytes * h as u32;

        let bmi = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: bpp,
            biCompression: 0,
            biSizeImage: pixel_size,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let mut pixel_ptr: *mut u8 = std::ptr::null_mut();
        let hbmp = CreateDIBSection(
            mem_dc,
            &bmi as *const _ as *const std::ffi::c_void,
            0,
            &mut pixel_ptr,
            std::ptr::null_mut(),
            0,
        );

        if hbmp.is_null() || pixel_ptr.is_null() {
            DeleteDC(mem_dc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            DestroyIcon(hicon);
            return Err("CreateDIBSection failed".to_string());
        }

        let old_bmp = SelectObject(mem_dc, hbmp);

        DrawIconEx(mem_dc, 0, 0, hicon, w, h, 0, std::ptr::null_mut(), 0x0003);

        SelectObject(mem_dc, old_bmp);

        // Read raw BGRA pixels, then encode as PNG (preserves alpha transparency)
        let pixel_count = (row_bytes * h as u32) as usize;
        let bgra = std::slice::from_raw_parts(pixel_ptr, pixel_count);

        // Convert BGRA -> RGBA (swap R/B)
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            let row_start = (y as u32 * row_bytes) as usize;
            for x in 0..w {
                let i = row_start + (x * 4) as usize;
                rgba.push(bgra[i + 2]); // R
                rgba.push(bgra[i + 1]); // G
                rgba.push(bgra[i]); // B
                rgba.push(bgra[i + 3]); // A
            }
        }

        use image::codecs::png::PngEncoder;
        use image::ColorType;
        use image::ImageEncoder;
        let mut png_bytes: Vec<u8> = Vec::new();
        {
            let encoder = PngEncoder::new(&mut png_bytes);
            encoder
                .write_image(&rgba, w as u32, h as u32, ColorType::Rgba8.into())
                .map_err(|e| format!("PNG encode failed: {}", e))?;
        }

        SelectObject(mem_dc, old_bmp);
        DeleteObject(hbmp);
        DeleteDC(mem_dc);
        ReleaseDC(std::ptr::null_mut(), screen_dc);
        DestroyIcon(hicon);

        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
        Ok(b64)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_file_icon(_path: String) -> Result<String, String> {
    Err("OS icons only supported on Windows".to_string())
}

// ── File content preview (text, markdown, pdf, docx) ──

pub fn get_file_preview(path: String) -> Result<serde_json::Value, String> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let text_exts = [
        "txt",
        "md",
        "js",
        "ts",
        "rs",
        "py",
        "vue",
        "json",
        "xml",
        "yaml",
        "yml",
        "toml",
        "css",
        "scss",
        "html",
        "sh",
        "bat",
        "log",
        "ini",
        "cfg",
        "env",
        "gitignore",
        "c",
        "cpp",
        "h",
        "hpp",
        "java",
        "go",
        "rb",
        "php",
        "swift",
        "kt",
        "dart",
        "lua",
        "r",
        "pl",
        "sql",
        "bash",
        "zsh",
        "fish",
    ];

    if text_exts.contains(&ext.as_str()) {
        let content = std::fs::read_to_string(&path).map_err(|e| format!("Read failed: {}", e))?;
        let preview: String = content.chars().take(5000).collect();
        return Ok(serde_json::json!({
            "type": if ext == "md" { "markdown" } else { "text" },
            "content": preview,
            "ext": ext,
        }));
    }

    if ext == "pdf" {
        let bytes = std::fs::read(&path).map_err(|e| format!("Read failed: {}", e))?;
        if bytes.len() > 10 * 1024 * 1024 {
            return Err("File too large for preview".to_string());
        }
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        return Ok(serde_json::json!({
            "type": "pdf",
            "data": b64,
        }));
    }

    if ext == "docx" {
        use std::io::Read;
        let file = std::fs::File::open(&path).map_err(|e| format!("Open failed: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Zip failed: {}", e))?;
        let mut text = String::new();
        if let Ok(mut entry) = archive.by_name("word/document.xml") {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).ok();
            let mut in_tag = false;
            for c in xml.chars() {
                if c == '<' {
                    in_tag = true;
                } else if c == '>' {
                    in_tag = false;
                } else if !in_tag && !c.is_control() {
                    text.push(c);
                }
            }
        }
        let preview: String = text.chars().take(3000).collect();
        return Ok(serde_json::json!({
            "type": "text",
            "content": preview,
            "ext": "docx",
        }));
    }

    Err("Unsupported file type for preview".to_string())
}
