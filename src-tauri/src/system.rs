use crate::native_drag;
use base64::Engine;
#[cfg(target_os = "windows")]
use image::codecs::png::PngEncoder;
#[cfg(target_os = "windows")]
use image::ColorType;
#[cfg(target_os = "windows")]
use image::ImageEncoder;
use log;
use std::io::Read;
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
    const IMAGE_PREVIEW_MAX_SIZE: usize = 2 * 1024 * 1024;
    if bytes.len() > IMAGE_PREVIEW_MAX_SIZE {
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
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(serde_json::json!({
        "mime": mime,
        "data": b64
    }))
}

/// Read raw file bytes as base64 for Office document preview (docx/xlsx/pptx)
pub fn read_file_bytes(path: String) -> Result<String, String> {
    const OFFICE_PREVIEW_MAX_SIZE: usize = 20 * 1024 * 1024; // 20MB
    let metadata = std::fs::metadata(&path).map_err(|e| format!("Failed to stat file: {}", e))?;
    if metadata.len() > OFFICE_PREVIEW_MAX_SIZE as u64 {
        return Err("File too large for preview".to_string());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("Read failed: {}", e))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
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

// ── File content preview (text, markdown, pdf) ──

const PREVIEW_TEXT_MAX_BYTES: usize = 512 * 1024;
const PREVIEW_TEXT_CHARS: usize = 10000;

/// Safely truncate a string to at most `max_chars` chars at a valid UTF-8 boundary
fn truncate_to_chars(s: &str, max_chars: usize) -> &str {
    let mut char_count = 0;
    for (i, _) in s.char_indices() {
        if char_count >= max_chars {
            return &s[..i];
        }
        char_count += 1;
    }
    s
}

/// Check if the first N bytes of a file look like valid UTF-8 text
fn is_probably_text(data: &[u8]) -> bool {
    if data.is_empty() {
        return true; // empty file is text
    }
    let check_len = data.len().min(8192);
    let slice = &data[..check_len];
    // Count null bytes and non-printable control chars
    let mut nulls = 0usize;
    let mut controls = 0usize;
    for &b in slice.iter() {
        if b == 0 {
            nulls += 1;
        } else if b < 0x08 || (b > 0x0D && b < 0x20) {
            controls += 1;
        }
    }
    // If more than 0.5% null bytes, it's binary
    if nulls as f64 > check_len as f64 * 0.005 {
        return false;
    }
    // If more than 10% control chars, it's likely binary
    if controls as f64 > check_len as f64 * 0.1 {
        return false;
    }
    // Try UTF-8 validation on the slice
    if std::str::from_utf8(slice).is_err() {
        // One more chance: check if it's mostly ASCII
        let printable = slice
            .iter()
            .filter(|&&b| b >= 0x20 && b < 0x7F || b == b'\n' || b == b'\r' || b == b'\t')
            .count();
        if (printable as f64) < check_len as f64 * 0.85 {
            return false;
        }
    }
    true
}

/// Known binary extensions that should skip text detection
fn is_known_binary_ext(ext: &str) -> bool {
    matches!(
        ext,
        "pdf"
            | "zip"
            | "7z"
            | "rar"
            | "tar"
            | "gz"
            | "tgz"
            | "bz2"
            | "tbz2"
            | "xz"
            | "txz"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "bmp"
            | "svg"
            | "ico"
            | "mp3"
            | "mp4"
            | "avi"
            | "mov"
            | "mkv"
            | "wav"
            | "flac"
            | "ttf"
            | "otf"
            | "woff"
            | "woff2"
            | "eot"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "wasm"
            | "class"
            | "jar"
            | "war"
            | "pyc"
            | "pyo"
            | "docx"
            | "xlsx"
            | "pptx"
            | "doc"
            | "xls"
            | "ppt"
            | "iso"
            | "dmg"
            | "deb"
            | "rpm"
    )
}

pub fn get_file_preview(path: String) -> Result<serde_json::Value, String> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Known binary types: skip text detection (handled by frontend)
    if is_known_binary_ext(&ext) {
        return Err("Binary file".to_string());
    }

    // Check file size before reading
    let metadata = std::fs::metadata(&path).map_err(|e| format!("Failed to stat: {}", e))?;
    if metadata.len() > PREVIEW_TEXT_MAX_BYTES as u64 {
        return Err("File too large for preview".to_string());
    }

    // Smart detection: read first bytes and check if text
    let bytes = std::fs::read(&path).map_err(|e| format!("Read failed: {}", e))?;
    if !is_probably_text(&bytes) {
        return Err("Binary file".to_string());
    }

    let content = String::from_utf8_lossy(&bytes);
    let preview = truncate_to_chars(&content, PREVIEW_TEXT_CHARS).to_string();

    // Markdown: detect by extension OR common markdown patterns
    let is_md = if ext == "md" || ext == "mdx" {
        true
    } else {
        // Heuristic: starts with '#' or contains markdown links
        preview.starts_with('#') && preview.contains('[') && preview.contains("](")
    };

    Ok(serde_json::json!({
        "type": if is_md { "markdown" } else { "text" },
        "content": preview,
        "ext": ext,
    }))
}

// ── Archive content listing ──

#[derive(serde::Serialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

const ARCHIVE_MAX_ENTRIES: u64 = 10000;

pub fn list_archive_contents(path: String) -> Result<Vec<ArchiveEntry>, String> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let full_lower = path.to_lowercase();

    match ext.as_str() {
        "zip" => list_zip(&path),
        "7z" => list_7z(&path),
        "rar" => list_rar(&path),
        "tar" => list_tar_archive(&path),
        "gz" | "tgz" if full_lower.ends_with(".tar.gz") || full_lower.ends_with(".tgz") => {
            list_tar_gz(&path)
        }
        "bz2" | "tbz2" if full_lower.ends_with(".tar.bz2") || full_lower.ends_with(".tbz2") => {
            list_tar_bz2(&path)
        }
        "xz" | "txz" if full_lower.ends_with(".tar.xz") || full_lower.ends_with(".txz") => {
            list_tar_xz(&path)
        }
        _ => Err("Unsupported archive format".to_string()),
    }
}

fn list_zip(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Open: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Zip: {}", e))?;
    let mut result = Vec::new();
    for i in 0..archive.len() {
        if result.len() as u64 >= ARCHIVE_MAX_ENTRIES {
            break;
        }
        let entry = archive.by_index(i).map_err(|e| format!("Entry: {}", e))?;
        let name = entry.name().to_string();
        let is_dir = entry.is_dir();
        result.push(ArchiveEntry {
            name: name.clone(),
            path: name.trim_end_matches('/').to_string(),
            size: if is_dir { 0 } else { entry.size() },
            is_dir,
        });
    }
    Ok(result)
}

fn list_tar_entries<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
) -> Result<Vec<ArchiveEntry>, String> {
    let mut result = Vec::new();
    for entry in archive.entries().map_err(|e| format!("Tar: {}", e))? {
        if result.len() as u64 >= ARCHIVE_MAX_ENTRIES {
            break;
        }
        let entry = entry.map_err(|e| format!("Entry: {}", e))?;
        let p = entry.path().map_err(|e| format!("Path: {}", e))?;
        let path_str = p.to_string_lossy().to_string();
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());
        result.push(ArchiveEntry {
            name,
            path: path_str,
            size: entry.size(),
            is_dir: entry.header().entry_type().is_dir(),
        });
    }
    Ok(result)
}

fn list_tar_archive(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Open: {}", e))?;
    let mut archive = tar::Archive::new(file);
    list_tar_entries(&mut archive)
}

fn list_tar_gz(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    list_tar_entries(&mut archive)
}

fn list_tar_bz2(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Open: {}", e))?;
    let bz2 = bzip2::read::BzDecoder::new(file);
    let mut archive = tar::Archive::new(bz2);
    list_tar_entries(&mut archive)
}

fn list_tar_xz(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Open: {}", e))?;
    let xz = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(xz);
    list_tar_entries(&mut archive)
}

fn list_7z(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let sz = sevenz_rust::SevenZReader::open(path, "".into()).map_err(|e| format!("7z: {}", e))?;
    let mut result = Vec::new();
    for entry in sz.archive().files.iter() {
        if result.len() as u64 >= ARCHIVE_MAX_ENTRIES {
            break;
        }
        let is_dir = entry.is_directory();
        result.push(ArchiveEntry {
            name: entry.name.clone(),
            path: entry.name.clone(),
            size: if is_dir { 0 } else { entry.size },
            is_dir,
        });
    }
    Ok(result)
}

fn list_rar(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    // Try system `unrar` command first, then `7z`
    for cmd in &["unrar", "7z"] {
        let output = std::process::Command::new(cmd)
            .args(if *cmd == "7z" {
                vec!["l", "-slt", path]
            } else {
                vec!["l", path]
            })
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                return parse_rar_output(cmd, &String::from_utf8_lossy(&out.stdout));
            }
        }
    }
    Err("RAR listing requires 'unrar' or '7z' installed".to_string())
}

fn parse_rar_output(cmd: &str, output: &str) -> Result<Vec<ArchiveEntry>, String> {
    let mut result = Vec::new();
    for line in output.lines() {
        if result.len() as u64 >= ARCHIVE_MAX_ENTRIES {
            break;
        }
        if cmd == "7z" {
            // 7z -slt output: Path = xxx, Size = xxx, Attributes = D...
            if let Some(p) = line.strip_prefix("Path = ") {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                result.push(ArchiveEntry {
                    name: p.to_string(),
                    path: p.to_string(),
                    size: 0, // will be filled by next Size line
                    is_dir: false,
                });
            }
            if let Some(s) = line.strip_prefix("Size = ") {
                if let Some(last) = result.last_mut() {
                    last.size = s.trim().parse().unwrap_or(0);
                }
            }
            if line.starts_with("Attributes = ") && line.contains('D') {
                if let Some(last) = result.last_mut() {
                    last.is_dir = true;
                    last.size = 0;
                }
            }
        } else {
            // unrar output: skip header lines, parse file entries
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3
                && !line.starts_with("UNRAR")
                && !line.starts_with("----")
                && !line.starts_with("  ")
                && !line.is_empty()
            {
                // unrar format has attributes, size, date, time, name
                let is_dir = parts[0].contains('d') || parts[0].contains('D');
                let name_start = line
                    .find(|c: char| {
                        c.is_ascii_alphabetic() && !c.is_ascii_uppercase() || c == '.' || c == '/'
                    })
                    .unwrap_or(0);
                let name = line[name_start..].trim().to_string();
                let size: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                if !name.is_empty() {
                    result.push(ArchiveEntry {
                        name: name.clone(),
                        path: name,
                        size,
                        is_dir,
                    });
                }
            }
        }
    }
    Ok(result)
}

// ── Extract single entry from archive for preview ──

const EXTRACT_MAX_SIZE: u64 = 20 * 1024 * 1024;

#[derive(serde::Serialize)]
pub struct ExtractResult {
    pub temp_path: String,
    pub original_name: String,
}

fn preview_temp_dir() -> std::path::PathBuf {
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("files-explorer-preview");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn safe_entry_path(entry: &str) -> Result<String, String> {
    let cleaned = entry.replace('\\', "/").trim_start_matches('/').to_string();
    if cleaned.contains("..") {
        return Err("Invalid path".to_string());
    }
    Ok(cleaned)
}

pub fn extract_archive_entry(
    archive_path: String,
    entry_path: String,
) -> Result<ExtractResult, String> {
    let safe_entry = safe_entry_path(&entry_path)?;
    let ext = std::path::Path::new(&archive_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let full_lower = archive_path.to_lowercase();
    let temp_dir = preview_temp_dir();
    let out_path = temp_dir.join(&safe_entry);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {}", e))?;
    }
    match ext.as_str() {
        "zip" => extract_zip_entry(&archive_path, &entry_path, &out_path),
        "7z" => extract_7z_entry(&archive_path, &entry_path, &out_path),
        "rar" => extract_rar_entry(&archive_path, &entry_path, &out_path),
        "tar" => extract_tar_entry(&archive_path, &entry_path, &out_path, None),
        "gz" | "tgz" if full_lower.ends_with(".tar.gz") || full_lower.ends_with(".tgz") => {
            extract_tar_entry(&archive_path, &entry_path, &out_path, Some("gz"))
        }
        "bz2" | "tbz2" if full_lower.ends_with(".tar.bz2") || full_lower.ends_with(".tbz2") => {
            extract_tar_entry(&archive_path, &entry_path, &out_path, Some("bz2"))
        }
        "xz" | "txz" if full_lower.ends_with(".tar.xz") || full_lower.ends_with(".txz") => {
            extract_tar_entry(&archive_path, &entry_path, &out_path, Some("xz"))
        }
        _ => Err("Unsupported archive format".to_string()),
    }?;
    let name = std::path::Path::new(&entry_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| entry_path.clone());
    Ok(ExtractResult {
        temp_path: out_path.to_string_lossy().to_string(),
        original_name: name,
    })
}

fn extract_zip_entry(archive: &str, entry: &str, out: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| format!("Open: {}", e))?;
    let mut za = zip::ZipArchive::new(file).map_err(|e| format!("Zip: {}", e))?;
    let idx = (0..za.len())
        .find(|i| {
            if let Ok(e) = za.by_index(*i) {
                let n = e.name().trim_end_matches('/');
                n == entry || n == entry.trim_start_matches('/')
            } else {
                false
            }
        })
        .ok_or_else(|| format!("Entry not found: {}", entry))?;
    let mut ze = za.by_index(idx).map_err(|e| format!("Entry: {}", e))?;
    if ze.size() > EXTRACT_MAX_SIZE {
        return Err("File too large".to_string());
    }
    let mut out_file = std::fs::File::create(out).map_err(|e| format!("Create: {}", e))?;
    std::io::copy(&mut ze, &mut out_file).map_err(|e| format!("Copy: {}", e))?;
    Ok(())
}

fn extract_tar_entry(
    archive: &str,
    entry: &str,
    out: &std::path::Path,
    comp: Option<&str>,
) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| format!("Open: {}", e))?;
    let mut tar: tar::Archive<Box<dyn Read>> = match comp {
        Some("gz") => tar::Archive::new(Box::new(flate2::read::GzDecoder::new(file))),
        Some("bz2") => tar::Archive::new(Box::new(bzip2::read::BzDecoder::new(file))),
        Some("xz") => tar::Archive::new(Box::new(xz2::read::XzDecoder::new(file))),
        _ => tar::Archive::new(Box::new(file)),
    };
    for e in tar.entries().map_err(|e| format!("Tar: {}", e))? {
        let mut e = e.map_err(|e| format!("Entry: {}", e))?;
        let p = e.path().map_err(|e| format!("Path: {}", e))?;
        let ps = p.to_string_lossy().to_string();
        if ps == entry || ps == format!("/{}", entry) || ps == entry.trim_start_matches('/') {
            if e.size() > EXTRACT_MAX_SIZE {
                return Err("File too large".to_string());
            }
            let mut of = std::fs::File::create(out).map_err(|e| format!("Create: {}", e))?;
            std::io::copy(&mut e, &mut of).map_err(|e| format!("Copy: {}", e))?;
            return Ok(());
        }
    }
    Err(format!("Not found: {}", entry))
}

fn extract_7z_entry(archive: &str, entry: &str, out: &std::path::Path) -> Result<(), String> {
    let mut sz =
        sevenz_rust::SevenZReader::open(archive, "".into()).map_err(|e| format!("7z: {}", e))?;
    sz.for_each_entries(|e, reader| {
        if e.name == entry || e.name == format!("/{}", entry) {
            if e.size > EXTRACT_MAX_SIZE {
                return Err(sevenz_rust::Error::Other(std::borrow::Cow::Borrowed(
                    "too large",
                )));
            }
            let mut of = std::fs::File::create(out)
                .map_err(|_| sevenz_rust::Error::Other(std::borrow::Cow::Borrowed("io")))?;
            std::io::copy(reader, &mut of).ok();
        }
        Ok(true)
    })
    .map_err(|e| format!("7z: {}", e))?;
    Ok(())
}

fn extract_rar_entry(archive: &str, entry: &str, out: &std::path::Path) -> Result<(), String> {
    for cmd in &["7z", "unrar"] {
        let out_dir = out.parent().unwrap_or(out).to_string_lossy().to_string();
        let args: Vec<String> = if *cmd == "7z" {
            vec![
                "e".into(),
                archive.into(),
                format!("-o{}", out_dir),
                entry.into(),
                "-y".into(),
            ]
        } else {
            vec!["x".into(), "-o+".into(), archive.into(), entry.into()]
        };
        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if let Ok(o) = std::process::Command::new(cmd).args(&str_args).output() {
            if o.status.success() {
                return Ok(());
            }
        }
    }
    Err("RAR extraction requires 'unrar' or '7z'".to_string())
}
