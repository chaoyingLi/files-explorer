use crate::error::{FsError, FsResult};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

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
    let file = fs::File::create(dest_path)
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
        if let Ok(entries) = fs::read_dir(path) {
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
        if let Ok(entries) = fs::read_dir(src) {
            for e in entries.flatten() {
                add_to_zip(zip, &e.path(), &relative, options, processed, total, app, cancel)?;
            }
        }
    } else {
        let name = relative.to_string_lossy().to_string();
        zip.start_file(&name, options)
            .map_err(|e| FsError::IoError(format!("Start file: {}", e)))?;
        let mut f = fs::File::open(src)
            .map_err(|e| FsError::IoError(format!("Open: {}", e)))?;
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
    let file =
        fs::File::open(archive).map_err(|e| FsError::NotFound(format!("Archive: {}", e)))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| FsError::IoError(format!("Open zip: {}", e)))?;
    let total = zip.len() as u64;

    for i in 0..zip.len() {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        let mut entry = zip
            .by_index(i)
            .map_err(|e| FsError::IoError(format!("Read entry: {}", e)))?;
        let out_path = Path::new(dest_dir).join(entry.name());
        if entry.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out = fs::File::create(&out_path)
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
    let file =
        fs::File::open(archive).map_err(|e| FsError::NotFound(format!("Archive: {}", e)))?;
    let mut tar = tar::Archive::new(file);
    let entries: Vec<_> = tar
        .entries()
        .map_err(|e| FsError::IoError(format!("Read tar: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();
    let total = entries.len() as u64;

    for (i, mut entry) in entries.into_iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        let out_path = Path::new(dest_dir).join(entry.path().unwrap_or_default());
        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out = fs::File::create(&out_path)
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
    let file =
        fs::File::open(archive).map_err(|e| FsError::NotFound(format!("Archive: {}", e)))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    let entries: Vec<_> = tar
        .entries()
        .map_err(|e| FsError::IoError(format!("Read tar.gz: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();
    let total = entries.len() as u64;

    for (i, mut entry) in entries.into_iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(FsError::Other("Cancelled".into()));
        }
        let out_path = Path::new(dest_dir).join(entry.path().unwrap_or_default());
        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out = fs::File::create(&out_path)
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
