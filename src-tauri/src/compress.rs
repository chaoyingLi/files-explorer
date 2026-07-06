use crate::core::error::{FsError, FsResult};
use crate::core::fs_helper;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// Sanitize archive entry paths to prevent Zip Slip / path traversal attacks.
/// Returns the cleaned path string, or an error if the path is invalid.
fn sanitize_archive_path(entry_path: &str) -> FsResult<String> {
    // Normalize separators and strip leading separators
    let cleaned = entry_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    // Reject any path containing ".." component
    if cleaned.split('/').any(|c| c == "..") {
        return Err(FsError::InvalidPath(format!(
            "Archive entry path contains illegal traversal: {}",
            entry_path
        )));
    }
    // Reject absolute paths (e.g. /etc/passwd or C:\\windows)
    if cleaned.starts_with('/')
        || cleaned.starts_with("\\\\")
        || cleaned.chars().nth(1) == Some(':')
    {
        return Err(FsError::InvalidPath(format!(
            "Archive entry path is absolute: {}",
            entry_path
        )));
    }
    Ok(cleaned)
}

/// Maximum total bytes to extract from an archive (2 GB)
const EXTRACT_TOTAL_MAX: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub current: u64,
    pub total: u64,
    pub file: String,
}

/// Compress files/folders into a zip archive
pub fn compress_zip(
    app: AppHandle,
    paths: Vec<String>,
    dest: String,
    cancel: Arc<AtomicBool>,
) -> FsResult<()> {
    let dest_path = Path::new(&dest);
    let file = fs_helper::file_create(dest_path)
        .map_err(|e| FsError::IoError(format!("Cannot create archive: {}", e)))?;
    let mut zip_writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut total_files: u64 = 0;
    let mut processed: u64 = 0;

    // Count total files first
    for src in &paths {
        count_files(Path::new(src), &mut total_files);
    }

    for src in &paths {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        add_to_zip(
            &mut zip_writer,
            Path::new(src),
            Path::new(""),
            options,
            &mut processed,
            total_files,
            &app,
            &cancel,
        )?;
    }

    zip_writer
        .finish()
        .map_err(|e| FsError::IoError(format!("Finalize zip: {}", e)))?;
    let _ = app.emit("compress-done", true);
    Ok(())
}

fn count_files(path: &Path, count: &mut u64) {
    if path.is_dir() {
        if let Ok(entries) = fs_helper::read_dir(path) {
            for e in entries.flatten() {
                count_files(&e.path(), count);
            }
        }
    } else {
        *count += 1;
    }
}

fn add_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    src: &Path,
    base: &Path,
    options: zip::write::SimpleFileOptions,
    processed: &mut u64,
    total: u64,
    app: &AppHandle,
    cancel: &Arc<AtomicBool>,
) -> FsResult<()> {
    if cancel.load(Ordering::SeqCst) {
        return Err(FsError::Other("Cancelled".into()));
    }

    let relative = base.join(src.file_name().unwrap_or_default());
    if src.is_dir() {
        let dir_name = relative.to_string_lossy().to_string() + "/";
        zip.add_directory(&dir_name, options)
            .map_err(|e| FsError::IoError(format!("Add dir: {}", e)))?;
        if let Ok(entries) = fs_helper::read_dir(src) {
            for e in entries.flatten() {
                add_to_zip(
                    zip,
                    &e.path(),
                    &relative,
                    options,
                    processed,
                    total,
                    app,
                    cancel,
                )?;
            }
        }
    } else {
        let name = relative.to_string_lossy().to_string();
        zip.start_file(&name, options)
            .map_err(|e| FsError::IoError(format!("Start file: {}", e)))?;
        let mut f =
            fs_helper::file_open(src).map_err(|e| FsError::IoError(format!("Open: {}", e)))?;
        let mut buf = [0u8; 8192];
        loop {
            let n = f
                .read(&mut buf)
                .map_err(|e| FsError::IoError(format!("Read: {}", e)))?;
            if n == 0 {
                break;
            }
            zip.write_all(&buf[..n])
                .map_err(|e| FsError::IoError(format!("Write: {}", e)))?;
        }
        *processed += 1;
        let _ = app.emit(
            "compress-progress",
            ProgressPayload {
                current: *processed,
                total,
                file: name,
            },
        );
    }
    Ok(())
}

/// Extract a zip/tar archive
pub fn extract_archive(
    app: AppHandle,
    archive: String,
    dest_dir: String,
    cancel: Arc<AtomicBool>,
) -> FsResult<()> {
    let path = Path::new(&archive);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "zip" => extract_zip(app, &archive, &dest_dir, cancel),
        "tar" => extract_tar(app, &archive, &dest_dir, cancel),
        "gz" | "tgz" => extract_tar_gz(app, &archive, &dest_dir, cancel),
        _ => Err(FsError::InvalidPath(format!(
            "Unsupported format: .{}",
            ext
        ))),
    }
}

fn extract_zip(
    app: AppHandle,
    archive: &str,
    dest_dir: &str,
    cancel: Arc<AtomicBool>,
) -> FsResult<()> {
    let file = fs_helper::file_open(Path::new(archive))
        .map_err(|e| FsError::NotFound(format!("Archive: {}", e)))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| FsError::IoError(format!("Open zip: {}", e)))?;
    let total = zip.len() as u64;

    let mut total_extracted: u64 = 0;

    for i in 0..zip.len() {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        let mut entry = zip
            .by_index(i)
            .map_err(|e| FsError::IoError(format!("Read entry: {}", e)))?;
        let safe_name = sanitize_archive_path(entry.name())?;
        let out_path = Path::new(dest_dir).join(&safe_name);
        if entry.is_dir() {
            fs_helper::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs_helper::create_dir_all(parent).ok();
            }
            // Check total size limit
            if entry.size() > EXTRACT_TOTAL_MAX
                || total_extracted + entry.size() > EXTRACT_TOTAL_MAX
            {
                return Err(FsError::Other(
                    "Archive too large: total extraction size exceeds limit".into(),
                ));
            }
            total_extracted += entry.size();
            let mut out = fs_helper::file_create(&out_path)
                .map_err(|e| FsError::IoError(format!("Create: {}", e)))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| FsError::IoError(format!("Extract: {}", e)))?;
        }
        let _ = app.emit(
            "compress-progress",
            ProgressPayload {
                current: (i + 1) as u64,
                total,
                file: entry.name().to_string(),
            },
        );
    }
    let _ = app.emit("compress-done", true);
    Ok(())
}

fn extract_tar(
    app: AppHandle,
    archive: &str,
    dest_dir: &str,
    cancel: Arc<AtomicBool>,
) -> FsResult<()> {
    let file = fs_helper::file_open(Path::new(archive))
        .map_err(|e| FsError::NotFound(format!("Archive: {}", e)))?;
    let mut tar = tar::Archive::new(file);
    let entries: Vec<_> = tar
        .entries()
        .map_err(|e| FsError::IoError(format!("Read tar: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();
    let total = entries.len() as u64;

    let mut total_extracted: u64 = 0;

    for (i, mut entry) in entries.into_iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        let raw_path = entry
            .path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let safe_name = sanitize_archive_path(&raw_path)?;
        let out_path = Path::new(dest_dir).join(&safe_name);
        if entry.header().entry_type().is_dir() {
            fs_helper::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs_helper::create_dir_all(parent).ok();
            }
            if total_extracted + entry.size() > EXTRACT_TOTAL_MAX {
                return Err(FsError::Other(
                    "Archive too large: total extraction size exceeds limit".into(),
                ));
            }
            total_extracted += entry.size();
            let mut out = fs_helper::file_create(&out_path)
                .map_err(|e| FsError::IoError(format!("Create: {}", e)))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| FsError::IoError(format!("Extract: {}", e)))?;
        }
        let _ = app.emit(
            "compress-progress",
            ProgressPayload {
                current: (i + 1) as u64,
                total,
                file: out_path.to_string_lossy().to_string(),
            },
        );
    }
    let _ = app.emit("compress-done", true);
    Ok(())
}

fn extract_tar_gz(
    app: AppHandle,
    archive: &str,
    dest_dir: &str,
    cancel: Arc<AtomicBool>,
) -> FsResult<()> {
    let file = fs_helper::file_open(Path::new(archive))
        .map_err(|e| FsError::NotFound(format!("Archive: {}", e)))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    let entries: Vec<_> = tar
        .entries()
        .map_err(|e| FsError::IoError(format!("Read tar.gz: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();
    let total = entries.len() as u64;

    let mut total_extracted: u64 = 0;

    for (i, mut entry) in entries.into_iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        let raw_path = entry
            .path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let safe_name = sanitize_archive_path(&raw_path)?;
        let out_path = Path::new(dest_dir).join(&safe_name);
        if entry.header().entry_type().is_dir() {
            fs_helper::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs_helper::create_dir_all(parent).ok();
            }
            if total_extracted + entry.size() > EXTRACT_TOTAL_MAX {
                return Err(FsError::Other(
                    "Archive too large: total extraction size exceeds limit".into(),
                ));
            }
            total_extracted += entry.size();
            let mut out = fs_helper::file_create(&out_path)
                .map_err(|e| FsError::IoError(format!("Create: {}", e)))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| FsError::IoError(format!("Extract: {}", e)))?;
        }
        let _ = app.emit(
            "compress-progress",
            ProgressPayload {
                current: (i + 1) as u64,
                total,
                file: out_path.to_string_lossy().to_string(),
            },
        );
    }
    let _ = app.emit("compress-done", true);
    Ok(())
}

// ── Archive content listing ──

#[derive(Clone, serde::Serialize)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

const ARCHIVE_MAX_ENTRIES: u64 = 10000;

pub fn list_archive_contents(path: String) -> Result<Vec<ArchiveEntry>, String> {
    use std::path::Path;
    let ext = Path::new(&path)
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
            list_tar_gz_list(&path)
        }
        "bz2" | "tbz2" if full_lower.ends_with(".tar.bz2") || full_lower.ends_with(".tbz2") => {
            list_tar_bz2_list(&path)
        }
        "xz" | "txz" if full_lower.ends_with(".tar.xz") || full_lower.ends_with(".txz") => {
            list_tar_xz_list(&path)
        }
        _ => Err("Unsupported archive format".to_string()),
    }
}

fn list_zip(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = fs_helper::file_open(Path::new(path)).map_err(|e| format!("Open: {}", e))?;
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
    let file = fs_helper::file_open(Path::new(path)).map_err(|e| format!("Open: {}", e))?;
    let mut a = tar::Archive::new(file);
    list_tar_entries(&mut a)
}
fn list_tar_gz_list(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = fs_helper::file_open(Path::new(path)).map_err(|e| format!("Open: {}", e))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut a = tar::Archive::new(gz);
    list_tar_entries(&mut a)
}
fn list_tar_bz2_list(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = fs_helper::file_open(Path::new(path)).map_err(|e| format!("Open: {}", e))?;
    let bz2 = bzip2::read::BzDecoder::new(file);
    let mut a = tar::Archive::new(bz2);
    list_tar_entries(&mut a)
}
fn list_tar_xz_list(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = fs_helper::file_open(Path::new(path)).map_err(|e| format!("Open: {}", e))?;
    let xz = xz2::read::XzDecoder::new(file);
    let mut a = tar::Archive::new(xz);
    list_tar_entries(&mut a)
}
fn list_7z(path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let sz = sevenz_rust::SevenZReader::open(path, "".into()).map_err(|e| {
        let m = format!("{}", e);
        if m.contains("password") || m.contains("encrypted") {
            format!("Password-protected 7z: {}", m)
        } else {
            format!("7z: {}", m)
        }
    })?;
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
    use crate::utils::time::run_command_with_timeout;
    use std::time::Duration;
    const T: Duration = Duration::from_secs(30);
    for cmd in &["unrar", "7z"] {
        let args: Vec<&str> = if *cmd == "7z" {
            vec!["l", "-slt", path]
        } else {
            vec!["l", path]
        };
        if let Ok(o) = run_command_with_timeout(cmd, &args, T) {
            if o.status.success() {
                return parse_rar_output(cmd, &String::from_utf8_lossy(&o.stdout));
            }
        }
    }
    Err("RAR listing requires 'unrar' or '7z'".into())
}
fn parse_rar_output(cmd: &str, output: &str) -> Result<Vec<ArchiveEntry>, String> {
    let mut result = Vec::new();
    for line in output.lines() {
        if result.len() as u64 >= ARCHIVE_MAX_ENTRIES {
            break;
        }
        if cmd == "7z" {
            if let Some(p) = line.strip_prefix("Path = ") {
                let p = p.trim();
                if !p.is_empty() {
                    result.push(ArchiveEntry {
                        name: p.to_string(),
                        path: p.to_string(),
                        size: 0,
                        is_dir: false,
                    });
                }
            }
            if let Some(s) = line.strip_prefix("Size = ") {
                if let Some(l) = result.last_mut() {
                    l.size = s.trim().parse().unwrap_or(0);
                }
            }
            if line.starts_with("Attributes = ") && line.contains('D') {
                if let Some(l) = result.last_mut() {
                    l.is_dir = true;
                    l.size = 0;
                }
            }
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3
                && !line.starts_with("UNRAR")
                && !line.starts_with("----")
                && !line.starts_with("  ")
                && !line.is_empty()
            {
                let is_dir = parts[0].contains('d') || parts[0].contains('D');
                let ns = line
                    .find(|c: char| {
                        c.is_ascii_alphabetic() && !c.is_ascii_uppercase() || c == '.' || c == '/'
                    })
                    .unwrap_or(0);
                let name = line[ns..].trim().to_string();
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

// ── Extract single entry ──

const EXTRACT_MAX_SIZE: u64 = 20 * 1024 * 1024;

#[derive(serde::Serialize)]
pub struct ExtractResult {
    pub temp_path: String,
    pub original_name: String,
}

pub fn extract_archive_entry(
    archive_path: String,
    entry_path: String,
) -> Result<ExtractResult, String> {
    let safe = entry_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    if safe.contains("..") {
        return Err("Invalid path".into());
    }
    use std::path::Path;
    let ext = Path::new(&archive_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let fl = archive_path.to_lowercase();
    let tmp = crate::platform::path_provider().temp_dir();
    let out = tmp.join(&safe);
    if let Some(p) = out.parent() {
        fs_helper::create_dir_all(p).map_err(|e| format!("{}", e))?;
    }
    match ext.as_str() {
        "zip" => ext_zip(&archive_path, &entry_path, &out),
        "7z" => ext_7z(&archive_path, &entry_path, &out),
        "rar" => ext_rar(&archive_path, &entry_path, &out),
        "tar" => ext_tar(&archive_path, &entry_path, &out, None),
        "gz" | "tgz" if fl.ends_with(".tar.gz") || fl.ends_with(".tgz") => {
            ext_tar(&archive_path, &entry_path, &out, Some("gz"))
        }
        "bz2" | "tbz2" if fl.ends_with(".tar.bz2") || fl.ends_with(".tbz2") => {
            ext_tar(&archive_path, &entry_path, &out, Some("bz2"))
        }
        "xz" | "txz" if fl.ends_with(".tar.xz") || fl.ends_with(".txz") => {
            ext_tar(&archive_path, &entry_path, &out, Some("xz"))
        }
        _ => Err("Unsupported archive format".into()),
    }?;
    let name = Path::new(&entry_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| entry_path);
    Ok(ExtractResult {
        temp_path: out.to_string_lossy().to_string(),
        original_name: name,
    })
}

fn ext_zip(a: &str, entry: &str, out: &Path) -> Result<(), String> {
    let f = fs_helper::file_open(Path::new(a)).map_err(|x| format!("{}", x))?;
    let mut za = zip::ZipArchive::new(f).map_err(|x| format!("{}", x))?;
    let idx = (0..za.len())
        .find(|i| {
            if let Ok(ze) = za.by_index(*i) {
                let n = ze.name().trim_end_matches('/');
                n == entry || n == entry.trim_start_matches('/')
            } else {
                false
            }
        })
        .ok_or_else(|| format!("Not found: {}", entry))?;
    let mut ze = za.by_index(idx).map_err(|x| format!("{}", x))?;
    if ze.size() > EXTRACT_MAX_SIZE {
        return Err("File too large".into());
    }
    let mut of = fs_helper::file_create(out).map_err(|x| format!("{}", x))?;
    std::io::copy(&mut ze, &mut of).map_err(|x| format!("{}", x))?;
    Ok(())
}

fn ext_tar(a: &str, e: &str, out: &Path, comp: Option<&str>) -> Result<(), String> {
    use std::io::Read;
    let f = fs_helper::file_open(Path::new(a)).map_err(|x| format!("{}", x))?;
    let mut t: tar::Archive<Box<dyn Read>> = match comp {
        Some("gz") => tar::Archive::new(Box::new(flate2::read::GzDecoder::new(f))),
        Some("bz2") => tar::Archive::new(Box::new(bzip2::read::BzDecoder::new(f))),
        Some("xz") => tar::Archive::new(Box::new(xz2::read::XzDecoder::new(f))),
        _ => tar::Archive::new(Box::new(f)),
    };
    for entry in t.entries().map_err(|x| format!("{}", x))? {
        let mut entry = entry.map_err(|x| format!("{}", x))?;
        let p = entry.path().map_err(|x| format!("{}", x))?;
        let ps = p.to_string_lossy().to_string();
        if ps == e || ps == format!("/{}", e) || ps == e.trim_start_matches('/') {
            if entry.size() > EXTRACT_MAX_SIZE {
                return Err("File too large".into());
            }
            let mut of = fs_helper::file_create(out).map_err(|x| format!("{}", x))?;
            std::io::copy(&mut entry, &mut of).map_err(|x| format!("{}", x))?;
            return Ok(());
        }
    }
    Err(format!("Not found: {}", e))
}

fn ext_7z(a: &str, e: &str, out: &Path) -> Result<(), String> {
    let mut sz = sevenz_rust::SevenZReader::open(a, "".into()).map_err(|x| {
        let m = format!("{}", x);
        if m.contains("password") || m.contains("encrypted") {
            format!("7z(pw):{}", m)
        } else {
            format!("7z:{}", m)
        }
    })?;
    sz.for_each_entries(|entry, reader| {
        if entry.name == e || entry.name == format!("/{}", e) {
            if entry.size > EXTRACT_MAX_SIZE {
                return Err(sevenz_rust::Error::Other(std::borrow::Cow::Borrowed(
                    "too large",
                )));
            }
            let mut of = fs_helper::file_create(out)
                .map_err(|_| sevenz_rust::Error::Other(std::borrow::Cow::Borrowed("io")))?;
            std::io::copy(reader, &mut of).ok();
        }
        Ok(true)
    })
    .map_err(|x| format!("{}", x))?;
    Ok(())
}

fn ext_rar(a: &str, e: &str, out: &Path) -> Result<(), String> {
    use crate::utils::time::run_command_with_timeout;
    use std::time::Duration;
    const T: Duration = Duration::from_secs(60);
    for cmd in &["7z", "unrar"] {
        let od = out.parent().unwrap_or(out).to_string_lossy().to_string();
        let args: Vec<String> = if *cmd == "7z" {
            vec![
                "e".into(),
                a.into(),
                format!("-o{}", od),
                e.into(),
                "-y".into(),
            ]
        } else {
            vec!["x".into(), "-o+".into(), a.into(), e.into()]
        };
        let sa: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if let Ok(o) = run_command_with_timeout(cmd, &sa, T) {
            if o.status.success() {
                return Ok(());
            }
        }
    }
    Err("RAR extraction requires 'unrar' or '7z'".into())
}
